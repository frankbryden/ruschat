// Create WebSocket connection.
const socket = new WebSocket("ws://localhost:9001");
const messageInput = document.getElementById("message-input");
const sendBtn = document.getElementById("send-button");
const chatWindow = document.getElementById("chat-window");
let username = "";

document.getElementById('login-button').addEventListener('click', function() {
    username = document.getElementById('username').value.trim();
    if (username) {
        document.querySelector('.login-view').classList.remove('active');
        document.querySelector('.chat-view').classList.add('active');
        socket.send(`user:${username}`);
    }
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


sendBtn.onclick = () => {
    const message = messageInput.value;
    console.log(`sending ${message}`);
    messageInput.value = "";
    addMessage(username, message);
    socket.send(message);
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
    const message = parts[1].trim();

    addMessage(user, message);
});
console.log("[1]");

console.log(sendBtn);