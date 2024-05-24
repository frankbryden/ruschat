// Create WebSocket connection.
const socket = new WebSocket("ws://localhost:9001");
const usernameInput = document.getElementById('username');
const messageInput = document.getElementById("message-input");
const sendBtn = document.getElementById("send-button");
const chatWindow = document.getElementById("chat-window");
const userList = document.getElementById('user-list');
let username = "";

function login() {
    username = usernameInput.value.trim();
    if (username) {
        document.querySelector('.login-view').classList.remove('active');
        document.querySelector('.chat-view').classList.add('active');
        socket.send(`user:${username}`);
    }
}

function sendMessage() {
    const message = messageInput.value;
    if (message.length > 0) {
        console.log(`sending ${message}`);
        messageInput.value = "";
        addMessage(username, message);
        socket.send(message);
    }
}

messageInput.onkeydown = event => {
    if (event.key == "Enter") {
        sendMessage();
    }
};

usernameInput.onkeydown = event => {
    if (event.key == "Enter") {
        login();
    }
};

document.getElementById('login-button').addEventListener('click', function() {
    login();
});

document.addEventListener('DOMContentLoaded', function() {
    document.querySelector('.login-view').classList.add('active');
});

function addMessage(user, message){
    console.log(`Creating message from user: ${user} and contents: ${message}`);
    // Div to hold message
    let messageDiv = document.createElement("div");
    messageDiv.classList.add("message");

    //Span to hold username
    let userSpan = document.createElement("span")
    userSpan.classList.add("user");
    userSpan.innerText = user + ": ";

    //Add text and user to message div
    messageDiv.innerText = message;
    messageDiv.insertAdjacentElement("afterbegin", userSpan);

    console.log(userSpan);
    console.log(messageDiv);

    // Add the result to the chat window
    chatWindow.appendChild(messageDiv);
}

function user_move_message(username, joined = true) {
    // Show join message
    const joinMessage = document.createElement('div');
    joinMessage.className = 'message join-message';
    if (joined) {
        joinMessage.textContent = `${username} joined the chat`;
    } else {
        joinMessage.textContent = `${username} left the chat`;
    }
    chatWindow.appendChild(joinMessage);
}

function render_lobby(users) {
    //Clear lobby
    userList.innerHTML = "";

    for (let user of users) {
        // Add user to user list
        const userItem = document.createElement('li');
        userItem.textContent = user;
        userList.appendChild(userItem);
    }
}


sendBtn.onclick = () => {
    sendMessage();
}
console.log("[0]");
// Connection opened
socket.addEventListener("open", (event) => {
});

// Listen for messages
socket.addEventListener("message", (event) => {
    console.log("Message from server ", event.data);
    const parts = event.data.split(":");
    const user = parts[0];
    if (user == "login") {
        user_move_message(parts[1], true);
        render_lobby(parts[2].split(","));
    } else if (user == "logout") {
        user_move_message(parts[1], false);
        render_lobby(parts[2].split(","));
    } else if (user == "lobby") {
        render_lobby(parts[1].split(","));
    } else {
        const message = parts[1].trim();
    
        addMessage(user, message);
    }
});
console.log("[1]");

console.log(sendBtn);