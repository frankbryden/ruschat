from websockets import client
import asyncio
from time import time

endpoint = "ws://localhost:9001"

async def test_load(iterations = 100):
    async with client.connect(endpoint) as websocket:
        await websocket.send("user:test#")
        start = time()
        for i in range(iterations):
            await websocket.send(f"Hey {i}")
        end = time()
        print(f"Performed {iterations} in {end - start}s (avg of {iterations/(end-start)} it/s) ")
        await websocket.close()

if __name__ == "__main__" :
    asyncio.run(test_load(iterations=1_000_000))