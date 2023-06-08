/*
 Message structure

 message {
    msg_type: 
        "join", "leave", -- User joining or leaving
        "update", -- User Updating their name
        "userlist", -- Userlist
        "message", "heartbeat", "error",
    sender: "username"
    color: "#ffffff" -- user color
    content: "message content"
 }
*/

const currentDate = new Date();

// Username variable
let username;
let usercolor = "#ffffff";

// Chat containers
const chat_container = document.querySelector("#chat_container");
const chat_userlist = document.querySelector("#user_container");

// Inputs
const message_field = document.querySelector("#input_message");
const message_send = document.querySelector("#input_send");

const color_selector = document.querySelector("#user_color");

// Websocket URL
const host = window.location.hostname;
const websocket_url = "ws://" + host + "/websocket";
const local = "ws://0.0.0.0:8080/websocket";

// Heartbeat timeout in ms
const heartbeat_timeout = 10000;

/* Join button */
join_btn.addEventListener("click", function(e) {
    if (join_username.value.length < 3) {
        PushAlert("Username should be atleast 3 characters long");
        return;
    }

    username = join_username.value;

    // Hide join screen, show chat
    SetVisible("#chat_app");
    SetHidden("#chat_join");

    const websocket = new WebSocket(local);

    websocket.onopen = function (e) {
        console.log("Socket connection opened");

        let reply = SocketMessage("join", username, usercolor, "joined");
        websocket.send(reply);
    }

    websocket.onclose = function(e) {
        PushAlert("Connection to the chat server was closed!");
        
        // Show join screen so user can try to reconnect
        SetVisible("#chat_join");
        return;
    }

    setInterval(() => {
        let reply = SocketMessage("heartbeat", username, usercolor, "beep boop");
        websocket.send(reply);
    }, heartbeat_timeout);

    websocket.onmessage = function(e) {
        let msg = JSON.parse(e.data);

        console.log("Received message => " + JSON.stringify(msg));

        if (msg.msg_type === "join" 
            || msg.msg_type === "leave" 
            || msg.msg_type === "update") {
            ChatNotification(msg);
        }
        else if (msg.msg_type === "message") {
            ChatMessage(msg);
        }
        else if (msg.msg_type === "userlist") {
            UpdateUsers(msg);
        }
        else if (msg.msg_type === "error") {
            PushAlert(msg.content);
        }
        else {
            console.log("Unknown message type? = " + msg.msg_type);
        }
    }

    function SendMessage() {
        let reply = SocketMessage("message", username, usercolor, message_field.value);
        message_field.value = '';

        websocket.send(reply);
    }

    message_field.onkeydown = function(e) {
        if (e.key == 'Enter') {
            SendMessage();
        }
    }

    message_send.addEventListener("click", function(e) {
        SendMessage();
    });
});

// Color button
color_btn.addEventListener("click", function(e) {
    console.log("Color button clicked");
    usercolor = color_selector.value;
});


/* UTILS */

function SocketMessage(msg_type, sender, color, message) {
    return JSON.stringify({
        "msg_type": msg_type,
        "sender": sender,
        "color": color,
        "content": message
    })
}

function ChatNotification(msg) {
    let template = 
    `
    <div class="flex flex-row justify-between content-center items-center bg-sky-950 hover:bg-sky-900 pl-5 pt-4 pb-4">
        <p class="font-extrabold text-[#ffffff]">Server : </p>
        <p class="break-words">` + msg.sender + ` ` + msg.content + `</p>
        <p class="text-right pr-2 text-xs text-gray-400">`+ GetTimestamp() + `</p>
    </div>
    `;

    chat_container.innerHTML += template;
}

function ChatMessage(msg) {
    let template =
    `
    <div class="flex flex-col bg-gray-800 hover:bg-gray-700 pl-5 pb-2">
        <h2 class="font-bold text-[` + msg.color + `]">` + msg.sender + ` </h2>
        <p class="pl-5 break-words pr-3">` + msg.content + ` </p>
        <p class="text-right pr-2 text-xs text-gray-400">` + GetTimestamp() + `</p>
    </div>
    `;

    chat_container.innerHTML += template;
}

function UpdateUsers(msg) {
    /*
    while(chat_userlist.hasChildNodes()) {
        chat_userlist.removeChild(userlist.firstChild);
    }
    */

    chat_userlist.innerHTML = "";

    msg.content.split("\n").forEach(function (item) {
        AddUser(item);
    });
}

// TODO: Implement colors for userlist?
// function AddUser(user_color, user_name) {
// <p class="text-[` + user_color + `]">` + user_name + `</p>

function AddUser(user_name) {
    let template =
    `
    <p class="text-fuchsia-500">` + user_name + `</p>
    `;

    chat_userlist.innerHTML += template;
}

function GetTimestamp() {
    let date = new Date();
    var time = date.getHours() + ":" + date.getMinutes() + ":" + date.getSeconds();
    return time;
}

function SetVisible(div_id) {
    let el = document.querySelector(div_id);
    el.classList.remove("hidden");
}

function SetHidden(div_id) {
    let el = document.querySelector(div_id);
    el.classList.add("hidden");
}

function PushAlert(message) {
    let alert_template = 
`
    <div class="text-pink-300 px-6 py-4 border-0 z-20 rounded absolute w-full mb-4 bg-violet-700">
    <span class="text-xl inline-block mr-5 align-middle">
        <svg fill="#ff0000" height="24" width="24" version="1.1" id="Capa_1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" 
            viewBox="0 0 310.806 310.806" xml:space="preserve">
            <path d="M305.095,229.104L186.055,42.579c-6.713-10.52-18.172-16.801-30.652-16.801c-12.481,0-23.94,6.281-30.651,16.801
            L5.711,229.103c-7.145,11.197-7.619,25.39-1.233,37.042c6.386,11.647,18.604,18.883,31.886,18.883h238.079
            c13.282,0,25.5-7.235,31.888-18.886C312.714,254.493,312.24,240.301,305.095,229.104z M155.403,253.631
            c-10.947,0-19.82-8.874-19.82-19.82c0-10.947,8.874-19.821,19.82-19.821c10.947,0,19.82,8.874,19.82,19.821
            C175.223,244.757,166.349,253.631,155.403,253.631z M182.875,115.9l-9.762,65.727c-1.437,9.675-10.445,16.353-20.119,14.916
            c-7.816-1.161-13.676-7.289-14.881-14.692l-10.601-65.597c-2.468-15.273,7.912-29.655,23.185-32.123
            c15.273-2.468,29.655,7.912,32.123,23.185C183.284,110.192,183.268,113.161,182.875,115.9z"/>
        </svg>
    </span>
    <span class="inline-block align-middle mr-8">
      <b class="capitalize">Error!</b> ` + message + `
    </span>
    <button class="absolute bg-transparent text-2xl font-semibold leading-none right-0 top-0 mt-4 mr-6 outline-none focus:outline-none" onclick="closeAlert(event)">
      <span>×</span>
    </button>
  </div>
`;  
    let alert_container = document.querySelector('#error_container');
    alert_container.innerHTML += alert_template;
}

function closeAlert(event) {
    let element = event.target;
    while(element.nodeName !== "BUTTON"){
      element = element.parentNode;
    }
    element.parentNode.parentNode.removeChild(element.parentNode);
}