from typing import Dict

import pytest

GPT_MODEL = "gpt-4o-2024-05-13"
GPT_EMBEDDINGS_MODEL = "text-embedding-ada-002"
LLAMA3_MODEL = "llama3:instruct"
OSS_EMBEDDINGS_MODEL = "nomic-embed-text"

school_1_description = """Stanford University is a private university located in Stanford, California, United States.
It was founded in 1885 by Leland Stanford and his wife, Jane, in memory of their only child, Leland Stanford Jr.
The university is ranked #5 in the world by QS World University Rankings.
It has over 17,000 students in total, that includes over 8,000 undergraduates and 9,000 graduates."""
student_1_description = """David Nguyen is a sophomore majoring in Computer Science at Stanford University.
He is Asian American and has a 3.8 GPA.
David is known for his programming skills and is an active member of the university's Robotics Club.
He hopes to pursue a career in artificial intelligence after graduating."""


def mark_skip() -> Dict[str, pytest.MarkDecorator]:
  return {"marks": pytest.mark.skip("Not Implemented yet")}
  # return {}


def mark_bodhi() -> Dict[str, pytest.MarkDecorator]:
  return {"marks": pytest.mark.bodhi()}

def mark_ollama() -> Dict[str, pytest.MarkDecorator]:
  return {"marks": pytest.mark.bodhi()}


def mark_bodhi_skip() -> Dict[str, pytest.MarkDecorator]:
  # return {"marks": [pytest.mark.bodhi(), pytest.mark.skip("Not Implemented yet")]}
  return mark_bodhi()


def mark_openai() -> Dict[str, pytest.MarkDecorator]:
  return {"marks": pytest.mark.openai()}


def mark_openai_skip() -> Dict[str, pytest.MarkDecorator]:
  return {"marks": [pytest.mark.openai(), pytest.mark.skip("Not Implemented yet")]}
