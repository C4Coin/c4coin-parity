import EventEmitter from 'eventemitter3';
const { ipcRenderer } = window.require('electron');

const METHOD_REQUEST_TOKEN = 'shell_requestNewToken';

class IpcProvider extends EventEmitter {
  constructor (appId) {
    super();
    this._appId = appId;
    this.id = 0;
    this._messages = {};
    this._queued = [];

    ipcRenderer.on('PARITY_IPC_CHANNEL', this.receiveMessage);
  }

  _constructMessage (id, data) {
    return Object.assign({}, data, {
      id,
      to: 'shell',
      from: this._appId,
      token: this._token
    });
  }

  receiveMessage = (_, { id, error, from, to, token, result }) => {
    const isTokenValid = token
      ? token === this._token
      : true;

    if (from !== 'shell' || to !== this._appId || !isTokenValid) {
      return;
    }

    if (this._messages[id].subscription) {
      // console.log('subscription', result, 'initial?', this._messages[id].initial);
      this._messages[id].initial
        ? this._messages[id].resolve(result)
        : this._messages[id].callback(error && new Error(error), result);
      this._messages[id].initial = false;
    } else {
      this._messages[id].callback(error && new Error(error), result);
      this._messages[id] = null;
    }
  }

  requestNewToken = () => new Promise((resolve, reject) => {
    // Webview is ready when receivin the ping
    ipcRenderer.once('ping', () => {
      this.send(METHOD_REQUEST_TOKEN, [], (error, token) => {
        if (error) {
          reject(error);
        } else {
          this.setToken(token);
          resolve(token);
        }
      });
    });
  })

  _send = (message) => {
    if (!this._token && message.data.method !== METHOD_REQUEST_TOKEN) {
      this._queued.push(message);

      return;
    }

    const id = ++this.id;
    const postMessage = this._constructMessage(id, message.data);

    this._messages[id] = Object.assign({}, postMessage, message.options);

    ipcRenderer.sendToHost('parity', { data: postMessage });
  }

  send (method, params, callback) {
    this._send({
      data: {
        method,
        params
      },
      options: {
        callback
      }
    });
  }

  _sendQueued () {
    if (!this._token) {
      return;
    }

    this._queued.forEach(this._send);
    this._queued = [];
  }

  setToken (token) {
    if (token) {
      this._connected = true;
      this._token = token;
      this.emit('connected');
      this._sendQueued();
    }
  }

  subscribe (api, callback, params) {
    // console.log('paritySubscribe', JSON.stringify(params), api, callback);
    return new Promise((resolve, reject) => {
      this._send({
        data: {
          api,
          params
        },
        options: {
          callback,
          resolve,
          reject,
          subscription: true,
          initial: true
        }
      });
    });
  }

  // FIXME: Should return callback, not promise
  unsubscribe (subId) {
    return new Promise((resolve, reject) => {
      this._send({
        data: {
          subId
        },
        options: {
          callback: (error, result) => {
            error
              ? reject(error)
              : resolve(result);
          }
        }
      });
    });
  }

  unsubscribeAll () {
    return this.unsubscribe('*');
  }
}

export default IpcProvider;
