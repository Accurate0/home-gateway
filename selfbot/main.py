import discord
import os
from contextlib import asynccontextmanager
import asyncio
from pydantic import BaseModel
from fastapi import FastAPI, status
import random

phrases = [
    "i do lachlan's bidding",
    "lachlan is my hero",
    "why...",
]


class MessageRequest(BaseModel):
    message: str
    channel_id: int


class DiscordClient(discord.Client):
    async def on_ready(self):
        status = discord.Status.idle
        activity_content = random.choice(phrases)
        activity = discord.Activity(
            type=discord.ActivityType.custom, name=activity_content)
        await client.change_presence(activity=activity, status=status)
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
