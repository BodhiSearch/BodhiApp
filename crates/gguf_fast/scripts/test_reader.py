from gguf import GGUFReader


if __name__ == "__main__":
    reader = GGUFReader("tests/data/sample0_le.gguf")
    print(reader.get_field("general.architecture"))
