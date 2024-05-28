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

const USER_SEPARATOR = "&";

const CLEAR_USER_TYPING_DELAY_MS = 1000;

function login() {
    username = usernameInput.value.trim();
    if (username) {
        document.querySelector('.login-view').classList.remove('active');
        document.querySelector('.chat-view').classList.add('active');
        socket.send(`user:${username}#${profilePicBlob}`);
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

// Function to handle sending an image
function sendImage() {
    if (imageUploadBtn.files.length > 0) {
        const file = imageUploadBtn.files[0];
        const reader = new FileReader();

        reader.onload = function(event) {
            const message = `image[${event.target.result}]`;
            socket.send(message);
            // Add the image to the chat window
            addMessage(username, message);

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

function addMessage(user, message){
    // Div to hold message
    let messageDiv = document.createElement("div");
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
        messageDiv.innerText = message;
    }
    messageDiv.insertAdjacentElement("afterbegin", profilePicImg);
    messageDiv.insertAdjacentElement("afterbegin", userSpan);

    // Add the result to the chat window
    chatWindow.appendChild(messageDiv);

    // Scroll chat view to the bottom, so users can see new messages as they arrive
    scrollToBottom();
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

async function ingest_images(users) {
    let usernames = [];
    for (let user of users) {
        let parts = user.split("#");
        let username = parts[0];
        usernames.push(username);
        let image = parts[1];
        if (image.length > 0) {
            const blob = await construct_blob_from_b64(image);
            const url = URL.createObjectURL(blob);
            imagesMapping.set(username, url);
        }
    }
    return usernames;
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

function userTyping() {
    if (typingTimeout) {
        clearTimeout(typingTimeout);
    }
    typingTimeout = setTimeout(clearUserTyping, CLEAR_USER_TYPING_DELAY_MS);
    socket.send("typing:start");
}

/**
 * Clear the current "user is typing status"
 */
function clearUserTyping() {
    socket.send("typing:stop");
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

// Listen for messages
socket.addEventListener("message", async (event) => {
    const parts = event.data.split(":");
    const user = parts[0];
    if (user == "login") {
        const users_data = parts.slice(2).join(":").split(USER_SEPARATOR);
        user_move_message(parts[1], true);
        const usernames = await ingest_images(users_data);
        render_lobby(usernames);
    } else if (user == "logout") {
        const users_data = parts[2].split(USER_SEPARATOR);
        user_move_message(parts[1], false);
        const usernames = await ingest_images(users_data);
        render_lobby(usernames);
    } else if (user == "lobby") {
        const users_data = parts.slice(1).join(":").split(USER_SEPARATOR);
        const usernames = await ingest_images(users_data);
        render_lobby(usernames);
    } else if (user == "typing") {
        user_typing_status(parts[1].split(USER_SEPARATOR));
    } else {
        const message = parts.slice(1).join(":").trim();
        addMessage(user, message);
    }
});
