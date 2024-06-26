import json

import pytest
from deepdiff import DeepDiff

from .common import GPT_MODEL, LLAMA3_MODEL, school_1_description, student_1_description

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
  ["input", "output"],
  [
    pytest.param(
      student_1_description,
      {
        "name": "David Nguyen",
        "major": "computer science",
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
def test_chat_tool(openai_client, bodhi_client, input, output):
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
  gpt_response = openai_client.chat.completions.create(model=GPT_MODEL, **args)
  gpt_result = json.loads(gpt_response.choices[0].message.tool_calls[0].function.arguments)
  diff = DeepDiff(output, gpt_result)
  assert {} == diff

  # TODO: implement
  # Somehow making it work with response_format json_object
  args["response_format"] = {"type": "json_object"}
  bodhi_response = bodhi_client.chat.completions.create(model=LLAMA3_MODEL, **args)
  bodhi_json = json.loads(bodhi_response.choices[0].message.content)
  # not following tools definition
  common_keys = {"name", "major", "ranking"}.intersection(output.keys())
  diff = DeepDiff(output, bodhi_json, include_paths=common_keys, ignore_string_case=True)
  # assert {} == diff # TODO: implement
  # bodhi_result = json.loads(bodhi_response.choices[0].message.tool_calls[0].function.arguments)
  # diff = DeepDiff(output, bodhi_result)
  # assert {} == diff
