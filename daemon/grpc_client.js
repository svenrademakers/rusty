const grpc = require('@grpc/grpc-js');
const protoLoader = require('@grpc/proto-loader');
const packageDefinition = protoLoader.loadSync('/home/svenr/rusty/daemon/proto/flaunch.proto', {});
const flaunch = grpc.loadPackageDefinition(packageDefinition).flaunch;

const client = new flaunch.ScriptEngine('localhost:50051', grpc.credentials.createInsecure());

const wat = () => {
    console.log("svensson");
    let call = client.GetAll();

    call.on('data', function (response) {
        console.log(response.message);
    });

    call.on('end', function () {
        console.log('All Salaries have been paid');
    });
};

exports.wat = wat;