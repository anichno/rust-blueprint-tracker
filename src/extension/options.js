import * as common from "./common.js";

var socket = null;
var auth_id_text = document.getElementById("auth_id");
var user_name = document.getElementById("user_name");
var team_id_text = document.getElementById("team_id");
var color_text = document.getElementById("color");

common.login(function(sock, data) {
    socket = sock;
    chrome.storage.sync.get("authentication_id", function(data) {
        auth_id_text.value = data.authentication_id;
    });
    if ("team_id" in data) {
        team_id_text.value = data.team_id;
    }

    if ("user_name" in data) {
        user_name.value = data.user_name;
    }
    
    if ("color" in data) {
        color_text.value = data.color;
    }

    document.getElementById("overwrite_user_name_btn").onclick = function() {
        sock.send(JSON.stringify({update_name: user_name.value}));
    }

    document.getElementById("create_team_btn").onclick = function() {
        sock.send(JSON.stringify({create_team: true}));
    };

    document.getElementById("join_team_btn").onclick = function() {
        sock.send(JSON.stringify({join_team: team_id_text.value}));
    };

    document.getElementById("leave_team_btn").onclick = function() {
        sock.send(JSON.stringify({leave_team: true}));
    };

    document.getElementById("color_btn").onclick = function() {
        sock.send(JSON.stringify({color: color_text.value}));
    }

    sock.onmessage = function(event) {
        var data = JSON.parse(event.data);
        console.log(data);
        if ("team_id" in data) {
            team_id_text.value = data.team_id;
        }
        if ("user_name" in data) {
            user_name.value = data.user_name;
        }
        if ("color" in data) {
            color_text.value = data.color;
        }

    };
});