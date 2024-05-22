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
  ["client_key", "model", "input", "clzz", "output"],
  [
    pytest.param("openai", GPT_MODEL, student_1_description, Student, student_1_output, id="openai_student"),
    pytest.param("bodhi", LLAMA3_MODEL, student_1_description, Student, student_1_output, id="bodhi_student"),
    pytest.param("openai", GPT_MODEL, school_1_description, University, school_1_output, id="openai_univ"),
    pytest.param("bodhi", LLAMA3_MODEL, school_1_description, University, school_1_output, id="bodhi_univ"),
  ],
)
def test_instructor_fn(api_clients, client_key, model, input, clzz, output):
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
  client = api_clients[client_key]
  client = instructor.patch(client, mode=instructor.mode.Mode.JSON)
  gpt_result = client.chat.completions.create(model=model, **args, response_model=clzz, max_retries=3)
  diff = DeepDiff(clzz(**output).model_dump(), gpt_result.model_dump())
  assert {} == diff
