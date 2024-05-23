import instructor
import pytest
from deepdiff import DeepDiff
from instructor.mode import Mode
from pydantic import BaseModel, Field

from .common import GPT_MODEL, LLAMA3_MODEL, mark_bodhi, mark_openai, school_1_description, student_1_description


class Student(BaseModel):
  name: str = Field(..., description="Name of the person")
  major: str = Field(..., description="Major subject.")
  school: str = Field(..., description="The university name.")
  grades: float = Field(..., description="GPA of the student.")
  club: str = Field(..., description="School club for extracurricular activities. ")


class University(BaseModel):
  name: str = Field(..., description="Name of the person")
  ranking: int = Field(..., description="QS world ranking of the school.")
  country: str = Field(..., description="Country of the school.")
  no_of_students: int = Field(..., description="Number of students enrolled in the school.")


student_1_output = {
  "name": "David Nguyen",
  "major": "Computer Science",
  "school": "Stanford University",
  "grades": 3.8,
  "club": "Robotics Club",
}

school_1_output = {
  "name": "Stanford University",
  "ranking": 5,
  "country": "United States",
  "no_of_students": 17000,
}


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model"],
  [
    pytest.param("openai", GPT_MODEL, id="openai", **mark_openai()),
    pytest.param("bodhi", LLAMA3_MODEL, id="bodhi", **mark_bodhi()),
  ],
  indirect=["client"],
  ids=["openai", "bodhi"],
)
@pytest.mark.parametrize(
  "mode",
  [Mode.JSON, Mode.TOOLS, Mode.FUNCTIONS],
  ids=["json", "tools", "functions"],
)
@pytest.mark.parametrize(
  ["input", "clzz", "output"],
  [
    pytest.param(student_1_description, Student, student_1_output, id="student"),
    pytest.param(school_1_description, University, school_1_output, id="univ"),
  ],
)
def test_instructor_fn(client, model, mode, input, clzz, output):
  if model == LLAMA3_MODEL and mode in [Mode.TOOLS, Mode.FUNCTIONS]:
    pytest.skip("Not Implemented")
  args = {
    "seed": 42,
    "messages": [
      {
        "role": "system",
        "content": "You are a helpful assistant, specializing in extracting structured information from text.",
      },
      {
        "role": "user",
        "content": input,
      },
    ],
  }
  client = instructor.patch(client, mode=mode)
  gpt_result = client.chat.completions.create(model=model, **args, response_model=clzz, max_retries=3)
  diff = DeepDiff(clzz(**output).model_dump(), gpt_result.model_dump())
  assert {} == diff


@pytest.mark.asyncio
@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model"],
  [
    pytest.param("async_openai", GPT_MODEL, id="async_openai", **mark_openai()),
    pytest.param("async_bodhi", LLAMA3_MODEL, id="async_bodhi", **mark_bodhi()),
  ],
  indirect=["client"],
)
@pytest.mark.parametrize(
  "mode",
  [Mode.JSON, Mode.TOOLS, Mode.FUNCTIONS],
  ids=["json", "tools", "functions"],
)
@pytest.mark.parametrize(
  ["input", "clzz", "output"],
  [
    pytest.param(student_1_description, Student, student_1_output, id="student"),
    pytest.param(school_1_description, University, school_1_output, id="univ"),
  ],
)
async def test_instructor_fn_async(client, model, mode, input, clzz, output):
  if model == LLAMA3_MODEL and mode in [Mode.TOOLS, Mode.FUNCTIONS]:
    pytest.skip("Not Implemented")
  args = {
    "seed": 42,
    "messages": [
      {
        "role": "system",
        "content": "You are a helpful assistant, specializing in extracting structured information from text.",
      },
      {
        "role": "user",
        "content": input,
      },
    ],
  }
  client = instructor.patch(client, mode=mode)
  gpt_result = await client.chat.completions.create(model=model, **args, response_model=clzz, max_retries=3)
  diff = DeepDiff(clzz(**output).model_dump(), gpt_result.model_dump())
  assert {} == diff
