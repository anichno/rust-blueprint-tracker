function get_element_by_data_id(data_id) {
    for (ul of document.getElementsByClassName("tracker-blueprints edit")) {
        for (li of ul.getElementsByTagName("li")) {
            if (li.getAttribute("data-id") == data_id) {
                return li;
            }
        }
    }
}

function click_id(data_id) {
    for (ul of document.getElementsByClassName("tracker-blueprints edit")) {
        for (li of ul.getElementsByTagName("li")) {
            if (li.getAttribute("data-id") == data_id) {
                li.click();
                return;
            }
        }
    }
}

function set_bps_to_known(bp_ids) {
    for (ul of document.getElementsByClassName("tracker-blueprints edit")) {
        for (li of ul.getElementsByTagName("li")) {
            if (bp_ids.has(parseInt(li.getAttribute("data-id")))) {
                if (!li.classList.contains("selected")) {
                    console.log("selected", li.getAttribute("data-id"))
                    li.click();
                }
            } else if (li.classList.contains("selected")) {
                console.log("unselected", li.getAttribute("data-id"))
                li.click();
            }
        }
    }
}

var known_bps = new Set();

function get_known_bps() {
    var known_bps = new Set();
    for (ul of document.getElementsByClassName("tracker-blueprints edit")) {
        for (var li of ul.getElementsByTagName("li")) {
            if (li.classList.contains("selected")) {
                known_bps.add(parseInt(li.getAttribute("data-id")));
            }
        }
    }
    return known_bps;
}

function handle_item_click() {
     var id = parseInt(this.getAttribute("data-id"))
    console.log(Date.now(), "you clicked: " + id);
    if (known_bps.has(id)) {
        known_bps.delete(id);
        port.postMessage({lost_bp: id});
    } else {
        known_bps.add(id);
        port.postMessage({learned_bp: id});
    }

}

function register_click_notify() {
    for (ul of document.getElementsByClassName("tracker-blueprints edit")) {
        for (var li of ul.getElementsByTagName("li")) {
            li.addEventListener("click", handle_item_click);
        }
    }
}

var team_members = new Map();
var member_list = document.createElement("ul");
member_list.id = "member_list";

function update_member_list() {
    console.log("update_member_list", team_members);
    for (let [id, member] of team_members) {
        var member_item = document.createElement("li");

        var member_box = document.createElement("div");
        member_box.setAttribute("class", "member_color_box");
        member_box.style.backgroundColor = member.color;
        member_item.appendChild(member_box);

        if (member.name) {
            var member_text = document.createTextNode(member.name);
            member_item.appendChild(member_text);
        } else {
            var member_text = document.createTextNode(id);
            member_item.appendChild(member_text);
        }
        
        member_list.appendChild(member_item);
    }

    var tracker_page = document.getElementsByTagName("body")[0];
    tracker_page.appendChild(member_list);
}

function update_team_known_bps(team_bps) {
    console.log(team_bps);
    let bp_map = new Map();
    for (let tbps of team_bps) {
        if (bp_map.has(tbps.bp)) {
            var bp_arr = bp_map.get(tbps.bp);
        } else {
            var bp_arr = new Array();
            bp_map.set(tbps.bp, bp_arr);
        }
        let color = team_members.get(tbps.user_id).color;
        bp_arr.push(color);

    }
    console.log(bp_map);

    for (let [data_id, team_colors] of bp_map) {
        let bp_el = get_element_by_data_id(data_id);
        let color_list = document.createElement("ul");
        color_list.setAttribute("class", "known_bps_overlay");
        color_list.style.width = "fit-content";
        color_list.style.height = "fit-content";
        color_list.style.padding = "0";
        color_list.style.border = "0";
        color_list.style.margin = "0";
        for (color of team_colors) {
            let li = document.createElement("li");
            li.setAttribute("class", "known_bps_overlay");
            li.style.width = "fit-content";
            li.style.height = "fit-content";
            li.style.paddingTop = "0";
            li.style.paddingBottom = "0";
            li.style.borderBottom = "0";
            li.style.borderRight = "0";
            li.style.padding = "0";
            li.style.border = "0";
            li.style.margin = "0";
            li.style.display = "block";
            let box = document.createElement("div");
            box.setAttribute("class", "member_color_box");
            box.style.backgroundColor = color;
            box.style.padding = "0";
            box.style.margin = "0";
            box.style.border = "0";
            li.appendChild(box);
            color_list.appendChild(li);
        }
        bp_el.appendChild(color_list);
    }
}

function setsEqual(set1, set2) {
    if (set1.size != set2.size) {
        return false;
    }

    for (let val of set1) {
        if (!set2.has(val)) {
            return false;
        }
    }

    return true;
}

function localRemoteOverwrite(local_bps, remote_bps, callback) {
    var sync_banner = document.createElement("div");
    sync_banner.id = "sync_banner";

    var sync_shade = document.createElement("div");
    sync_shade.id = "sync_shade";
    sync_banner.appendChild(sync_shade);

    var buttons_span = document.createElement("span");
    buttons_span.style.display = "block";
    buttons_span.style.backgroundColor = "white";
    buttons_span.style.zIndex = "1000";
    buttons_span.style.position = "relative";
    buttons_span.style.top = "100";
    buttons_span.style.left = "25%";
    buttons_span.appendChild(document.createTextNode("You have a mismatch between your local and remote saved blueprints. Which would you like to keep?"));
    
    var local_btn = document.createElement("input");
    local_btn.type = "button";
    local_btn.value = "Local";
    local_btn.setAttribute("class", "sync_button");
    local_btn.onclick = function() {
        if (window.confirm("This will overwrite any remote known blueprints with the blueprints seleted on this page.")) {
            port.postMessage({clear_bp: true});
            for (let bp of local_bps) {
                // send learn_bp for bp
                port.postMessage({learned_bp: bp});
            }
            sync_banner.remove();
            callback(local_bps);
        }
    };
    buttons_span.appendChild(local_btn);
    
    var remote_btn = document.createElement("input");
    remote_btn.type = "button";
    remote_btn.value = "Remote";
    remote_btn.setAttribute("class", "sync_button");
    remote_btn.onclick = function() {
        if (window.confirm("This will overwrite any locally known blueprints with what the server has for your user.")) {
            callback(remote_bps);
            sync_banner.remove();
        }
    };
    buttons_span.appendChild(remote_btn);

    sync_banner.appendChild(buttons_span);
    
    var body_el = document.getElementsByTagName("body")[0];
    body_el.insertBefore(sync_banner, body_el.childNodes[0]);
}

document.getElementById("tracker-clear").addEventListener("click", function() {
    if (window.confirm("Clear blueprints from server?")) {
        port.postMessage({clear_bp: true});
    }
});

var port = chrome.runtime.connect({name: "rustlabs_messaging"});

chrome.runtime.onMessage.addListener(function(msg, sender) {
    console.log(Date.now(), msg);
    if ("known_bps" in msg) {
        var already_known_bps = get_known_bps();
        var newKnownBps = new Set(msg.known_bps);
        if (!setsEqual(already_known_bps, newKnownBps)) {
            console.log("remote-local mismatch");
            localRemoteOverwrite(already_known_bps, newKnownBps, function(now_known_bps) {
                set_bps_to_known(now_known_bps);
                known_bps = now_known_bps;
                register_click_notify();
            })
        } else {
            set_bps_to_known(newKnownBps);
            known_bps = newKnownBps;
            register_click_notify();
        }
    }
    if ("team_members" in msg) {
        for (let member of msg.team_members) {
            team_members.set(member.user_id, {name: member.user_name, color: member.color});
        }
        update_member_list();
    }
    if ("team_known_bps" in msg) {
        update_team_known_bps(msg.team_known_bps);
    }
})

