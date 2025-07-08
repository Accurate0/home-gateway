import discord
import os
from contextlib import asynccontextmanager
import asyncio
from pydantic import BaseModel
from fastapi import FastAPI, status


class MessageRequest(BaseModel):
    message: str
    channel_id: int


class DiscordClient(discord.Client):
    async def on_ready(self):
        print(f'Logged on as {self.user}!')


client = DiscordClient()


async def start_discord():
    await client.start(
        os.environ['DISCORD_TOKEN'])


@asynccontextmanager
async def lifespan(app: FastAPI):
    loop = asyncio.get_running_loop()
    loop.create_task(start_discord())
    yield


app = FastAPI(lifespan=lifespan)


@app.post("/message", status_code=status.HTTP_204_NO_CONTENT)
async def message(body: MessageRequest):
    channel = client.get_channel(body.channel_id)
    await channel.send(body.message)  # type: ignore


@app.get("/health", status_code=status.HTTP_204_NO_CONTENT)
async def health():
    return None
