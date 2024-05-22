import instructor
import pytest
from deepdiff import DeepDiff
from pydantic import BaseModel, Field

from .common import GPT_MODEL, LLAMA3_MODEL, school_1_description, student_1_description


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


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["input", "model", "output"],
  [
    (
      student_1_description,
      Student,
      {
        "name": "David Nguyen",
        "major": "Computer Science",
        "school": "Stanford University",
        "grades": 3.8,
        "club": "Robotics Club",
      },
    ),
    (
      school_1_description,
      University,
      {
        "name": "Stanford University",
        "ranking": 5,
        "country": "United States",
        "no_of_students": 17000,
      },
    ),
  ],
  ids=["chat_tool_student_1", "chat_tool_univ_1"],
)
def test_instructor_fn(openai_client, bodhi_client, input, model, output):
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

  openai_client = instructor.patch(openai_client, mode=instructor.mode.Mode.JSON)
  gpt_result = openai_client.chat.completions.create(model=GPT_MODEL, **args, response_model=model, max_retries=3)
  diff = DeepDiff(model(**output).model_dump(), gpt_result.model_dump())
  assert {} == diff

  bodhi_client = instructor.patch(bodhi_client, mode=instructor.mode.Mode.JSON)
  bodhi_result = bodhi_client.chat.completions.create(model=LLAMA3_MODEL, **args, response_model=model, max_retries=3)
  diff = DeepDiff(model(**output).model_dump(), bodhi_result.model_dump())
  assert {} == diff
