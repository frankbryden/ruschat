const BOM = new Uint8Array([0xEF,0xBB,0xBF]);

//Taken from https://stackoverflow.com/a/74827975/2538341
async function load_image(file, size) {
    console.log("Loading image");
    size ??= 256

    const canvas = document.createElement('canvas')
    const ctx = canvas.getContext('2d')

    canvas.width = size
    canvas.height = size

    const bitmap = await createImageBitmap(file)
    console.log("Created bitmap:");
    console.log(bitmap);
    const { width, height } = bitmap

    const ratio = Math.max(size / width, size / height)

    const x = (size - (width * ratio)) / 2
    const y = (size - (height * ratio)) / 2

    ctx.drawImage(bitmap, 0, 0, width, height, x, y, width * ratio, height * ratio)
    console.log("Drew image");

    return canvas.toDataURL("image/jpeg");
};

async function construct_blob_from_b64(b64) {
    // b64 will look something like "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAUA=="
    return fetch(b64)
        .then(res => res.blob());
}

// Function to get the current time in HH:MM format
function formatTimestamp(timestamp) {
    const hours = String(timestamp.getHours()).padStart(2, '0');
    const minutes = String(timestamp.getMinutes()).padStart(2, '0');
    return `${hours}:${minutes}`;
}