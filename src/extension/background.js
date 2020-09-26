import * as common from "./common.js";


// // Register on install
// chrome.runtime.onInstalled.addListener(function(details) {
//     console.log(details);
//     if (details.reason == "install") {
//         var socket = new WebSocket(common.server_addr);
//         socket.addEventListener("open", function(event) {
//             socket.send(JSON.stringify({register: true}));
//         });
//         socket.addEventListener("message", function(event) {
//             var result = JSON.parse(event.data);
//             console.log(event.data);
//             console.log(result);
//             console.log("new user id: " + result.authentication_id);
//             chrome.storage.sync.set({authentication_id: result.authentication_id});
//             socket.close();
//         });
//     }
// });

var socket = null;
var socket_active = false;
var logged_in = false;

chrome.webNavigation.onCompleted.addListener(function() {
    common.login(function(sock, data) {
        socket = sock;
        chrome.tabs.query({active: true, currentWindow: true}, function(tabs) {
            chrome.tabs.sendMessage(tabs[0].id, {known_bps: data.known_bps, team_members: data.team_members, team_known_bps: data.team_known_bps});
        });
    });
    // chrome.storage.sync.get("authentication_id", function(data) {
    //     init(data.authentication_id);
    //     console.log("Blueprint tracker initialized");
    // });

}, {url: [{urlMatches : 'https://rustlabs.com/blueprint-tracker'}, {urlMatches : chrome.runtime.getURL('/options.html')}]});



// function init(authentication_id) {
//     console.log("logging in with authentication id: " + authentication_id);

//     // Create WebSocket connection.
//     socket = new WebSocket(server_addr);

//     // Connection opened
//     socket.addEventListener('open', function (event) {
//         socket.send(JSON.stringify({login: authentication_id}));
//     });

//     // Listen for messages
//     socket.addEventListener('message', function (event) {
//         console.log('Message from server ', event.data);
//         var data = JSON.parse(event.data);
//         if ("authenticated" in data) {
//             if (data.authenticated) {
//                 console.log("Authentication Success");
//                 logged_in = true;

//                 chrome.tabs.query({active: true, currentWindow: true}, function(tabs) {
//                     chrome.tabs.sendMessage(tabs[0].id, {known_bps: data.known_bps});
//                 });

                
//             } else {
//                 console.log("Authentication Failed");
//             }
//         }
//     });
// }

chrome.runtime.onConnect.addListener(function(port) {
    console.assert(port.name == "rustlabs_messaging");
    console.log(Date.now(), "listening for rustlabs events");
    port.onMessage.addListener(function(msg) {
        console.log(msg);
        if ("learned_bp" in msg) {
            socket.send(JSON.stringify({learned_bp: msg.learned_bp}));
        } else if ("lost_bp" in msg) {
            socket.send(JSON.stringify({forgot_bp: msg.lost_bp}));
        } else if ("clear_bp" in msg) {
            socket.send(JSON.stringify({clear_bp: msg.clear_bp}));
        }
    });
});
