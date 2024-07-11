import time

import pytest
from openai import OpenAI

from tests.common import GPT_MODEL, LLAMA3_MODEL, mark_bodhi_skip, mark_openai


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model"],
  [
    pytest.param("openai", GPT_MODEL, id="openai", **mark_openai()),
    pytest.param("bodhi", LLAMA3_MODEL, id="bodhi", **mark_bodhi_skip()),
  ],
  indirect=["client"],
)
def test_assistant_example(client: OpenAI, model):
  assistant = client.beta.assistants.create(
    name="Math Tutor",
    instructions="You are a personal math tutor. Write and run code to answer math questions.",
    tools=[{"type": "code_interpreter"}],
    model=model,
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


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model"],
  [
    pytest.param("openai", GPT_MODEL, id="openai", **mark_openai()),
    pytest.param("bodhi", LLAMA3_MODEL, id="bodhi", **mark_bodhi_skip()),
  ],
  indirect=["client"],
)
def test_assistant_example_with_file(client: OpenAI, model):
  vector_store = client.beta.vector_stores.create(name="[Test] OpenAI API Resources")
  guide = client.files.create(file=open("tests/data/assistants-guide.md", "rb"), purpose="assistants")
  _ = client.beta.vector_stores.files.create_and_poll(vector_store_id=vector_store.id, file_id=guide.id)
  assistant = client.beta.assistants.create(
    name="OpenAI API Expert",
    instructions="You are a OpenAI API Expert. Refer to the knowledgebase to answer user queries.",
    tools=[{"type": "file_search"}],
    tool_resources={"file_search": {"vector_store_ids": [vector_store.id]}},
    model=GPT_MODEL,
  )
  thread = client.beta.threads.create(
    messages=[
      {
        "role": "user",
        "content": "What is a run step?",
      }
    ]
  )
  _ = client.beta.threads.runs.create_and_poll(
    thread_id=thread.id,
    assistant_id=assistant.id,
  )
  messages = client.beta.threads.messages.list(thread_id=thread.id)
  assert "list of actions that an Assistant takes" in messages.data[0].content[0].text.value
