// Create WebSocket connection.
const socket = new WebSocket("ws://localhost:9001");
const usernameInput = document.getElementById('username');
const profilePicInput = document.getElementById('profile-picture');
const profilePicElement = document.getElementById('profile-pic-img');
const messageInput = document.getElementById("message-input");
const sendBtn = document.getElementById("send-button");
const imageUploadBtn = document.getElementById("image-upload");
const chatWindow = document.getElementById("chat-window");
const userList = document.getElementById('user-list');
const typingStatusDiv = document.getElementById('typing-status');

const imageRegex = /^image\[(.*?)\]$/m;
let username = "";
let profilePicBlob = "";
let typingTimeout = null;
let profilePicDataUrl = ''; // Variable to hold the profile picture data URL
let imagesMapping = new Map();
let typing = false;
let topTimestamp = new Date();
let botTimestamp = new Date();
let topElement = null;

const USER_SEPARATOR = "&";

const CLEAR_USER_TYPING_DELAY_MS = 1000;

usernameInput.focus();

function login() {
    username = usernameInput.value.trim();
    if (username) {
        document.querySelector('.login-view').classList.remove('active');
        document.querySelector('.chat-view').classList.add('active');
        messageInput.focus();
        socket.send(`user:${username}#${profilePicBlob}`);
        requestHistory();
    }
}

function requestHistory() {
    socket.send(`history:${topTimestamp.getTime()}`);
}

function sendMessage() {
    const message = messageInput.value;
    if (message.length > 0) {
        console.log(`sending ${message}`); 
        messageInput.value = "";
        addMessage(new Date(), username, message);
        socket.send(message);
    }
}

// Function to handle sending an image
function sendImage() {
    if (imageUploadBtn.files.length > 0) {
        const file = imageUploadBtn.files[0];
        const reader = new FileReader();

        reader.onload = function(event) {
            const message = `image[${event.target.result}]`;
            socket.send(message);
            // Add the image to the chat window
            addMessage(new Date(), username, message);

            // Clear the image input
            imageUploadBtn.value = '';

            // Scroll to bottom
            scrollToBottom();
        };
        reader.readAsDataURL(file);
    }
} 

messageInput.onkeydown = event => {
    if (event.key == "Enter") {
        sendMessage();
        clearTimeout(typingTimeout);
        clearUserTyping();
    } else {
        userTyping();
    }
};

usernameInput.onkeydown = event => {
    if (event.key == "Enter") {
        login();
    }
};

profilePicInput.onchange = event => {
    const file = profilePicInput.files[0];
    const reader = new FileReader();
    reader.onload = async ev => {
        const img = new Image();
        console.log();
        img.src = ev.target.result;
        img.onload = async ev => {
            const base64Img = await load_image(img, 32);
            profilePicElement.src = base64Img;
            profilePicBlob = base64Img;
        }
    };
    reader.readAsDataURL(file);
}

document.getElementById('login-button').addEventListener('click', function() {
    login();
});

document.addEventListener('DOMContentLoaded', function() {
    document.querySelector('.login-view').classList.add('active');
});

function addMessage(timestamp, user, message){
    // Div to hold message
    const messageDiv = document.createElement("div");
    messageDiv.classList.add("message");

    //Profile pic
    let profilePicImg = document.createElement("img");
    profilePicImg.classList.add("profile-pic");
    profilePicImg.src = imagesMapping.get(user) ?? "static/blank-user-profile.png";

    //Span to hold username
    let userSpan = document.createElement("span")
    userSpan.classList.add("user");
    userSpan.innerText = user + ": ";

    const match = message.match(imageRegex);
    if (match) {
        const img = document.createElement("img");
        img.src = match[1];
        img.alt = "Sent Image";
        img.style = "max-width: 200px; border-radius: 8px; margin-top: 5px;";

        const br = document.createElement("br");

        //This is an image
        messageDiv.insertAdjacentElement("afterbegin", img);
        messageDiv.insertAdjacentElement("afterbegin", br);
    } else {
        //Add text and user to message div
        messageDiv.innerHTML = `${message}<span class="timestamp">${formatTimestamp(timestamp)}</span>`;
    }
    messageDiv.insertAdjacentElement("afterbegin", profilePicImg);
    messageDiv.insertAdjacentElement("afterbegin", userSpan);

    // Add the result to the chat window
    addContentToChatWindow(messageDiv, timestamp);
}

function userMoveMessage(timestamp, username, joined = true) {
    // Show join message
    const joinMessage = document.createElement('div');
    joinMessage.className = 'message join-message';
    if (joined) {
        joinMessage.textContent = `${username} joined the chat`;
    } else {
        joinMessage.textContent = `${username} left the chat`;
    }
    const timestampSpan = document.createElement('span');
    timestampSpan.classList.add("timestamp");
    timestampSpan.innerText = formatTimestamp(timestamp);

    joinMessage.append(timestampSpan);

    addContentToChatWindow(joinMessage, timestamp);
}

function addContentToChatWindow(node, timestamp) {
    if (timestamp < topTimestamp) {
        chatWindow.prepend(node);
        if (topElement) {
            topElement.scrollIntoView();
        }
        topTimestamp = timestamp;
    } else if (timestamp > botTimestamp) {
        chatWindow.append(node);
        botTimestamp = timestamp;
        // Scroll chat view to the bottom, so users can see new messages as they arrive
        scrollToBottom();
    } else {
        console.error(`Cannot insert message in the middle of the pile (${topTimestamp} < ${timestamp} < ${botTimestamp})`);
    }
}

function user_typing_status(typingUsers) {
    if (typingUsers.length > 1) {
        typingStatusDiv.textContent = `${typingUsers.join(", ")} are typing...`;
    } else if (typingUsers.length == 1 && typingUsers[0].length > 0) {
        typingStatusDiv.textContent = `${typingUsers[0]} is typing...`;
    } else {
        typingStatusDiv.textContent = "";
    }
}

function scrollToBottom() {
    chatWindow.scrollTop = chatWindow.scrollHeight;
}

async function ingestImages(users) {
    let usernames = [];
    console.log(users);
    for (let user of users) {
        let parts = user.split("#");
        let username = parts[0];
        usernames.push(username);
        let image = parts[1];
        if (image && image.length > 0) {
            const blob = await construct_blob_from_b64(image);
            const url = URL.createObjectURL(blob);
            imagesMapping.set(username, url);
        }
    }
    return usernames;
}

function renderLobby(users) {
    //Clear lobby
    userList.innerHTML = "";

    for (let user of users) {
        // Add user to user list
        const userItem = document.createElement('li');
        userItem.textContent = user;
        userList.appendChild(userItem);
    }
}

function userTyping() {
    if (typingTimeout) {
        clearTimeout(typingTimeout);
    }
    typingTimeout = setTimeout(clearUserTyping, CLEAR_USER_TYPING_DELAY_MS);
    if (!typing) {
        typing = true;
        socket.send("typing:start");
    }
}

/**
 * Clear the current "user is typing status"
 */
function clearUserTyping() {
    socket.send("typing:stop");
    typingTimeout = null;
    typing = false;
}

function handleScrollToTop() {
    topElement = chatWindow.children[0];
    requestHistory();
}


sendBtn.onclick = () => {
    sendMessage();
}

imageUploadBtn.onchange = () => {
    sendImage();
}

// Connection opened
socket.addEventListener("open", (event) => {
});

chatWindow.addEventListener('scroll', function() {
    if (chatWindow.scrollTop === 0) {
        handleScrollToTop();
    }
});

// Listen for messages
socket.addEventListener("message", async (event) => {
    const parts = event.data.split(":");
    const timestamp = new Date(parseInt(parts[0]));
    const user = parts[1];
    let usersData = null;
    if (user == "login") {
        userMoveMessage(timestamp, parts[2], true);
        usersData = parts.slice(3).join(":").split(USER_SEPARATOR);
    } else if (user == "logout") {
        userMoveMessage(timestamp, parts[2], false);
        usersData = parts[3].split(USER_SEPARATOR);
    } else if (user == "lobby") {
        usersData = parts.slice(2).join(":").split(USER_SEPARATOR);
    } else if (user == "typing") {
        user_typing_status(parts[2].split(USER_SEPARATOR));
    } else {
        const message = parts.slice(2).join(":").trim();
        addMessage(timestamp, user, message);
    }
    if (usersData && !(usersData.length == 1 && usersData[0].length == 0)) {
        const usernames = await ingestImages(usersData);
        renderLobby(usernames);
    }
});