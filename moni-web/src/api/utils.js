const { MoniRpcServiceClient } = require("./proto/moni-rpc_grpc_web_pb.js");

const RPC_CLIENT_ADDRESS = "http://127.0.0.1:38779";

function createClient() {
  let client = new MoniRpcServiceClient(RPC_CLIENT_ADDRESS);
  return client;
}

async function callClient(request, callFunc) {
  let client = createClient();
  let promise = new Promise((resolve, reject) => {
    let fn = client[callFunc];
    if (!fn) {
      reject("no such function");
      return;
    }
    client[callFunc](request, {}, (err, response) => {
      if (err) {
        resolve({ error: err });
        return;
      }
      resolve(response.toObject());
    });
  });
  return promise;
}

export { RPC_CLIENT_ADDRESS, callClient };
