export const server_addr = "ws://161.35.102.49:9090";

// callback(socket, account_info)
export function login(callback) {
    chrome.storage.sync.get("authentication_id", function(data) {
        if ("authentication_id" in data) {
            do_login(data.authentication_id, callback);
        } else {
            register(function(auth_id) {
                do_login(auth_id, callback);
            });
        }
    });
}

function register(callback) {
    var socket = new WebSocket(server_addr);
    socket.addEventListener("open", function(event) {
        socket.send(JSON.stringify({register: true}));
    });
    socket.addEventListener("message", function(event) {
        var result = JSON.parse(event.data);
        console.log(event.data);
        console.log(result);
        console.log("new user id: " + result.authentication_id);
        chrome.storage.sync.set({authentication_id: result.authentication_id}, callback(result.authentication_id));
        socket.close();
    });
}

function do_login(authentication_id, callback) {
    console.log("logging in with authentication id: " + authentication_id);

    // Create WebSocket connection.
    var socket = new WebSocket(server_addr);

    // Connection opened
    socket.onopen = function (event) {
        socket.send(JSON.stringify({login: authentication_id}));
        socket.onopen = null;
        socket.onmessage = function(event) {
            socket.onmessage = null;
            console.log('Message from server ', event.data);
            var data = JSON.parse(event.data);
            if ("authenticated" in data) {
                if (data.authenticated) {
                    console.log("Authentication Success");
                    // logged_in = true;
    
                    // chrome.tabs.query({active: true, currentWindow: true}, function(tabs) {
                    //     chrome.tabs.sendMessage(tabs[0].id, {known_bps: data.known_bps});
                    // });
                    callback(socket, data);
    
                    
                } else {
                    console.log("Authentication Failed");
                }
            }
        };
    };
}