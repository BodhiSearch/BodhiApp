import base64
import pytest
from openai import OpenAI

from tests.common import GPT_MODEL, LLAMA3_MODEL


def read_as_base64(path):
  with open(path, "rb") as file:
    encoded_content = base64.b64encode(file.read()).decode()
  return encoded_content


def request_image_args():
  image = read_as_base64("tests/data/chatgpt-ui.jpeg")
  return {
    "seed": 42,
    "messages": [
      {
        "role": "system",
        "content": "You are a helpful assistant. "
        "You can understand images of application screenshot and breakdown it into hierarchical components.",
      },
      {
        "role": "user",
        "content": [
          {
            "type": "image_url",
            "image_url": {"url": f"data:image/jpeg;base64,#{image}"},
          },
          {"type": "text", "text": "Describe the user interface in the uploaded image."},
        ],
      },
    ],
  }


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model"],
  [
    pytest.param("openai", GPT_MODEL, id="openai"),
    pytest.param("bodhi", LLAMA3_MODEL, id="bodhi", marks=pytest.mark.skip("Not implemented yet")),
  ],
  indirect=["client"],
)
def test_image_upload(client: OpenAI, model):
  args = dict(**request_image_args())
  response = client.chat.completions.create(model=model, **args)
  content = response.choices[0].message.content
  assert "ChatGPT" in content
