/*
Request structure

{
    msg_type: string
    senders: string,
    content: string
}
*/
// Joining chat server
const join_btn = document.querySelector('#join_btn');
const username_field = document.querySelector('#username');
let username;

// Notifications
const notifications = document.querySelector("#notifs");

// Chat fields
const chat_container = document.querySelector("#chat-container");
const message_field = document.querySelector('#message_field');
const message_btn = document.querySelector('#message_btn');
const chatwindow = document.querySelector('#chat-window');
const userlist = document.querySelector('#userlist-container');

// Websocket urls
const host = window.location.hostname;
const ws_url = "ws://" + host + ":8080/websocket";
const wss_url = "wss://" + host + "/websocket";
const localhost = "localhost:8080";

// Anti-timeout - Heartbeat - time in ms
const heartbeat_timeout = 10000;

// Join button handler
join_btn.addEventListener('click', function(e) {
    if (username_field.value.length < 3) {
        add_alert("Try a username of atleast 3 characters");
        return;
    }
    username = username_field.value;

    chat_container.attributes.removeNamedItem("hidden");

    const websock = new WebSocket(wss_url);

    websock.onopen = function (e) {
        console.log("Websocket connection established");
        join_btn.disabled = true;
        username_field.disabled = true;
        username_field.readOnly = true;
        
        let reply = build_reply("join", username, username);
        websock.send(reply);
    };

    websock.onclose = function () {
        console.log("Websocket connection closed");
        add_alert("Websocket connection closed");
        join_btn.disabled = false;
        username_field.disabled = false;
        username_field.readOnly = false;
        return;
    };

    setInterval(() => {
        let reply = build_reply("heartbeat", username, "beep boop");
        websock.send(reply);
    }, heartbeat_timeout);

    websock.onmessage = function(e) {
        let j = JSON.parse(e.data);

        console.log("Received message => " + JSON.stringify(j));

        if (j.msg_type === "join" || j.msg_type === "leave") {
            add_chat_notification(j);
        }

        if (j.msg_type === "message") {
            add_message(j);
        }

        if (j.msg_type === "userlist") {

            while(userlist.hasChildNodes()) {
                userlist.removeChild(userlist.firstChild);
            }

            j.content.split("\n").forEach(function (item) {
                add_user(item);
            });
        }

        if (j.msg_type == "invalid_username") {
            add_alert("Server: " + j.content);
        }
    }

    message_btn.addEventListener("click", function(e) {
        console.log("sending message");
        let reply = build_reply(
            "message", username, message_field.value
        );
        websock.send(reply);
    
        message_field.value = "";
    });
    
    message_field.onkeydown = function(e) {
        if (e.key == "Enter") {
            console.log("sending message");
            let reply = build_reply(
                "message", username, message_field.value
            );
            websock.send(reply);
    
            message_field.value = "";
        }
    }
    
});

// Utils
function add_user(user) {
    let div = document.createElement('div');
    div.className = "user-item";
    div.innerHTML = user;

    userlist.appendChild(div);
}

let last_bg = "chat_bg1";
let last_div_id = 0;
let last_sender = null;
function add_message(msg) {
    let bg = null;

    if (last_sender !== msg.sender) {
        if (last_bg == "chat_bg1") { bg = "chat_bg2"; last_bg = "chat_bg2"; }
        else { bg = "chat_bg1"; last_bg = "chat_bg1"; }
        // Different sender than last message
        let div = document.createElement('div');
        div.className = "chat_msg " + bg;
        div.id = "msg_" + (last_div_id + 1);
        let sender = document.createElement('div');
        sender.className = "chat_sender";
        sender.innerHTML = msg.sender + " : ";
        div.appendChild(sender);
        let message_d = document.createElement('div');
        message_d.className = "chat_content";
        message_d.innerHTML = msg.content;
        div.appendChild(message_d);
    
        last_div_id += 1;
        last_sender = msg.sender;
    
        chatwindow.appendChild(div);
    } else {
        console.log("same user");
        let div = document.querySelector("#msg_" + last_div_id);
        console.log(div);
        let message_d = document.createElement('div');
        message_d.className = "chat_content";
        message_d.innerHTML = msg.content;
        div.appendChild(message_d);
    }

    
    chatwindow.scrollTop = chatwindow.scrollHeight;
}

function add_chat_notification(notif) {
    let div = document.createElement('div');
    div.className = "chat_notice";

    if (notif.msg_type == "join") {
        div.innerHTML = "Server Notice: a new user joined! - " + notif.sender;
    } else if (notif.msg_type == "leave") {
        div.innerHTML = "Server Notice: a user left! - " + notif.sender;
    }

    chatwindow.appendChild(div);
    chatwindow.scrollTop = chatwindow.scrollHeight;
}

function add_alert(alert) {
    let div = document.createElement('div');
    div.className = "flex-column alert";
    div.innerHTML = alert;

    if (notifications.childElementCount >= 3) {
        console.log("Removing first notif");
        notifications.firstElementChild.remove();
    }
    notifications.appendChild(div);
}

function build_reply(kind, sender, message) {
    return JSON.stringify({
        "msg_type": kind,
        "sender": sender,
        "content": message
    });
}