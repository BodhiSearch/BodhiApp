import time

import pytest
from openai import OpenAI

from tests.common import GPT_MODEL, mark_bodhi_skip, mark_openai


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client"],
  [
    pytest.param("openai", id="openai", **mark_openai()),
    pytest.param("bodhi", id="bodhi", **mark_bodhi_skip()),
  ],
  indirect=["client"],
)
def test_assistant_example(client: OpenAI):
  assistant = client.beta.assistants.create(
    name="Math Tutor",
    instructions="You are a personal math tutor. Write and run code to answer math questions.",
    tools=[{"type": "code_interpreter"}],
    model=GPT_MODEL,
  )
  thread = client.beta.threads.create()
  _ = client.beta.threads.messages.create(
    thread_id=thread.id, role="user", content="I need to solve the equation `3x + 11 = 14`. Can you help me?"
  )
  run = client.beta.threads.runs.create_and_poll(
    thread_id=thread.id,
    assistant_id=assistant.id,
    instructions="Please address the user as Jane Doe. The user has a premium account.",
  )
  while run.status != "completed":
    time.sleep(0.5)
  messages = client.beta.threads.messages.list(thread_id=thread.id)
  assert "x = 1" in messages.data[0].content[0].text.value
