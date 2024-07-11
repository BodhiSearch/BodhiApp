import json

import pytest
from deepdiff import DeepDiff

from .common import GPT_MODEL, LLAMA3_MODEL, mark_bodhi, mark_openai, school_1_description, student_1_description

tools = [
  {
    "type": "function",
    "function": {
      "name": "extract_student_info",
      "description": "Get the student information from the body of the input text",
      "parameters": {
        "type": "object",
        "properties": {
          "name": {"type": "string", "description": "Name of the person"},
          "major": {"type": "string", "description": "Major subject."},
          "school": {"type": "string", "description": "The university name."},
          "grades": {"type": "integer", "description": "GPA of the student."},
          "club": {"type": "string", "description": "School club for extracurricular activities. "},
        },
      },
    },
  },
  {
    "type": "function",
    "function": {
      "name": "extract_school_info",
      "description": "Get the school information from the body of the input text",
      "parameters": {
        "type": "object",
        "properties": {
          "name": {"type": "string", "description": "Name of the school."},
          "ranking": {"type": "integer", "description": "QS world ranking of the school."},
          "country": {"type": "string", "description": "Country of the school."},
          "no_of_students": {"type": "integer", "description": "Number of students enrolled in the school."},
        },
      },
    },
  },
]

tool_params = {
  "n": 1,
  "seed": 42,
  "tool_choice": "auto",
  "tools": tools,
  "user": "user-1234",
}


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model"],
  [
    pytest.param("openai", GPT_MODEL, id="openai", **mark_openai()),
    pytest.param("bodhi", LLAMA3_MODEL, id="bodhi", **mark_bodhi()),
  ],
  indirect=["client"],
)
@pytest.mark.parametrize(
  ["input", "output"],
  [
    pytest.param(
      student_1_description,
      {
        "name": "David Nguyen",
        "major": "Computer Science",
        "school": "Stanford University",
        "grades": 3.8,
        "club": "Robotics Club",
      },
      id="chat_tool_student_1",
      marks=pytest.mark.unmarked(),
    ),
    pytest.param(
      school_1_description,
      {
        "name": "Stanford University",
        "ranking": 5,
        "country": "United States",
        "no_of_students": 17000,
      },
      id="chat_tool_univ_1",
      marks=pytest.mark.unmarked(),
    ),
  ],
)
def test_chat_tool(client, model, input, output):
  args = dict(**tool_params)
  args["messages"] = [
    {
      "role": "system",
      "content": "You are a helpful assistant, specializing in extracting structured information from text.",
    },
    {
      "role": "user",
      "content": input,
    },
  ]
  response = client.chat.completions.create(model=model, **args)
  result = json.loads(response.choices[0].message.tool_calls[0].function.arguments)
  diff = DeepDiff(output, result)
  assert {} == diff

weather_tools = [
  {
    "type": "function",
    "function": {
      "name": "get_current_weather",
      "description": "Get the current weather in a given location",
      "parameters": {
        "type": "object",
        "properties": {
          "location": {
            "type": "string",
            "description": "The city and state, e.g. San Francisco, CA",
          },
          "unit": {"type": "string", "enum": ["celsius", "fahrenheit"]},
        },
        "required": ["location"],
      },
    },
  }
]


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model"],
  [
    pytest.param("openai", GPT_MODEL, id="openai", **mark_openai()),
    pytest.param("bodhi", LLAMA3_MODEL, id="bodhi", **mark_bodhi()),
  ],
  indirect=["client"],
)
def test_chat_tool_simple(client, model):
  response = client.chat.completions.create(
    model=model,
    messages=[{"role": "user", "content": "What's the weather like in Boston today?"}],
    tools=weather_tools,
    tool_choice="auto",
  )
  choice = response.choices[0]
  assert choice.finish_reason == "tool_calls"
  assert choice.message.content is None
  function = choice.message.tool_calls[0].function
  assert json.loads(function.arguments) == {"location": "Boston, MA"}
  assert function.name == "get_current_weather"
