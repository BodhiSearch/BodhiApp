# coding=utf-8
# Copyright 2022 EleutherAI and the HuggingFace Inc. team. All rights reserved.
#
# This code is based on EleutherAI's GPT-NeoX library and the GPT-NeoX
# and OPT implementations in this library. It has been modified from its
# original forms to accommodate minor architectural differences compared
# to GPT-NeoX and OPT used by the Meta AI team that trained the model.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Tokenization classes for INFLMTokenizer."""
import os
from shutil import copyfile
from typing import Any, Dict, List, Optional, Tuple

import sentencepiece as spm

from transformers.tokenization_utils import PreTrainedTokenizer
from transformers.utils import logging

from tokenizers import pre_tokenizers,Regex,decoders
from tokenizers.pre_tokenizers import Digits, Split, ByteLevel
import os 
           
# same as gpt4 cl-base-100k
PATTERN = Regex("(?i:'s|'t|'re|'ve|'m|'ll|'d)|[^\r\n\p{L}\p{N}]?\p{L}+|\p{N}{1,3}| ?[^\s\p{L}\p{N}]+[\r\n]*|\s*[\r\n]+|\s+(?!\S)|\s+\s+(\S)+")
                
logger = logging.get_logger(__name__)

VOCAB_FILES_NAMES = {"vocab_file": "./tokenizer.model"}

PRETRAINED_VOCAB_FILES_MAP = {}

      
class INFLMTokenizer(PreTrainedTokenizer):
    """
    Construct a INFLMTokenizer tokenizer based on sentence-piece 

    Args:
        vocab_file (`str`):
            Path to the vocabulary file.
    """

    vocab_files_names = VOCAB_FILES_NAMES
    pretrained_vocab_files_map = PRETRAINED_VOCAB_FILES_MAP
    model_input_names = ["input_ids", "attention_mask"]
    _auto_class = "AutoTokenizer"

    def __init__(
        self,
        vocab_file,
        unk_token="<unk>",
        bos_token="<s>",
        eos_token="</s>",
        pad_token="<pad>",
        sp_model_kwargs: Optional[Dict[str, Any]] = None,
        add_bos_token=False,
        add_eos_token=False,
        decode_with_prefix_space=False,
        clean_up_tokenization_spaces=False,
        spaces_between_special_tokens=False,
        **kwargs,
    ):
        self.sp_model_kwargs = {} if sp_model_kwargs is None else sp_model_kwargs
        self.vocab_file = vocab_file
        self.add_bos_token = add_bos_token
        self.add_eos_token = add_eos_token
        self.decode_with_prefix_space = decode_with_prefix_space
        self.sp_model = spm.SentencePieceProcessor(**self.sp_model_kwargs)
        self.sp_model.Load(vocab_file)
        self._no_prefix_space_tokens = None
        self.pre_tokenizer = pre_tokenizers.Sequence([Split(pattern =PATTERN,behavior = "isolated", invert = False)])
        super().__init__(
            bos_token=bos_token,
            eos_token=eos_token,
            unk_token=unk_token,
            pad_token=pad_token,
            clean_up_tokenization_spaces=clean_up_tokenization_spaces,
            spaces_between_special_tokens=spaces_between_special_tokens,
            **kwargs,
        ) 

        """ Initialisation"""

    @property
    def no_prefix_space_tokens(self):
        if self._no_prefix_space_tokens is None:
            vocab = self.convert_ids_to_tokens(list(range(self.vocab_size)))
            self._no_prefix_space_tokens = {i for i, tok in enumerate(vocab) if not tok.startswith("▁")}
        return self._no_prefix_space_tokens

    @property
    def vocab_size(self):
        """Returns vocab size"""
        return self.sp_model.get_piece_size()

    @property
    def bos_token_id(self) -> Optional[int]:
        return self.sp_model.bos_id()

    @property
    def eos_token_id(self) -> Optional[int]:
        return self.sp_model.eos_id()

    def get_vocab(self):
        """Returns vocab as a dict"""
        vocab = {self.convert_ids_to_tokens(i): i for i in range(self.vocab_size)}
        vocab.update(self.added_tokens_encoder)
        return vocab

    def _tokenize(self, text):
        """Returns a tokenized string."""
        
        splits = self.pre_tokenizer.pre_tokenize_str(text)
        texts=[]
       
        for split in splits:
            texts.extend(self.sp_model.encode(split[0], out_type=str))
        return texts

    def _convert_token_to_id(self, token):
        """Converts a token (str) in an id using the vocab."""
       
        return self.sp_model.piece_to_id(token)

    def _convert_id_to_token(self, index):
        """Converts an index (integer) in a token (str) using the vocab."""
        token = self.sp_model.IdToPiece(index)
        return token

    def _maybe_add_prefix_space(self, tokens, decoded):
        if tokens and tokens[0] not in self.no_prefix_space_tokens:
            return " " + decoded
        else:
            return decoded

    def convert_tokens_to_string(self, tokens):
        """Converts a sequence of tokens (string) in a single string."""
        current_sub_tokens = []
        out_string = ""
        prev_is_special = False
        for token in tokens:
            # make sure that special tokens are not decoded using sentencepiece model
            if token in self.all_special_tokens:
                out_string += self.sp_model.decode(current_sub_tokens) + token
                prev_is_special = True
                current_sub_tokens = []
            else:
                current_sub_tokens.append(token)
                prev_is_special = False
        out_string += self.sp_model.decode(current_sub_tokens)
        
        return out_string

    def save_vocabulary(self, save_directory, filename_prefix: Optional[str] = None) -> Tuple[str]:
        """
        Save the vocabulary and special tokens file to a directory.

        Args:
            save_directory (`str`):
                The directory in which to save the vocabulary.

        Returns:
            `Tuple(str)`: Paths to the files saved.
        """
        if not os.path.isdir(save_directory):
            logger.error(f"Vocabulary path ({save_directory}) should be a directory")
            return
        out_vocab_file = os.path.join(
            save_directory, (filename_prefix + "-" if filename_prefix else "") + VOCAB_FILES_NAMES["vocab_file"]
        )

        if os.path.abspath(self.vocab_file) != os.path.abspath(out_vocab_file) and os.path.isfile(self.vocab_file):
            copyfile(self.vocab_file, out_vocab_file)
        elif not os.path.isfile(self.vocab_file):
            with open(out_vocab_file, "wb") as fi:
                content_spiece_model = self.sp_model.serialized_model_proto()
                fi.write(content_spiece_model)

        return (out_vocab_file,)

    def build_inputs_with_special_tokens(self, token_ids_0, token_ids_1=None):
        if self.add_bos_token:
            bos_token_ids = [self.bos_token_id]
        else:
            bos_token_ids = []

        output = bos_token_ids + token_ids_0

        if token_ids_1 is not None:
            output = output + token_ids_1

        if self.add_eos_token:
            output = output + [self.eos_token_id]

        return output

    def get_special_tokens_mask(
        self, token_ids_0: List[int], token_ids_1: Optional[List[int]] = None, already_has_special_tokens: bool = False
    ) -> List[int]:
        """
        Retrieve sequence ids from a token list that has no special tokens added. This method is called when adding
        special tokens using the tokenizer `prepare_for_model` method.

        Args:
            token_ids_0 (`List[int]`):
                List of IDs.
            token_ids_1 (`List[int]`, *optional*):
                Optional second list of IDs for sequence pairs.
            already_has_special_tokens (`bool`, *optional*, defaults to `False`):
                Whether or not the token list is already formatted with special tokens for the model.

        Returns:
            `List[int]`: A list of integers in the range [0, 1]: 1 for a special token, 0 for a sequence token.
        """
        if already_has_special_tokens:
            return super().get_special_tokens_mask(
                token_ids_0=token_ids_0, token_ids_1=token_ids_1, already_has_special_tokens=True
            )

        eos_token_id = [1] if self.add_eos_token else []
        if token_ids_1 is None:
            return  ([0] * len(token_ids_0)) + eos_token_id
        return  ([0] * len(token_ids_0)) + eos_token_id + ([0] * len(token_ids_1)) + eos_token_id
    

    def create_token_type_ids_from_sequences(
        self, token_ids_0: List[int], token_ids_1: Optional[List[int]] = None
    ) -> List[int]:
        """
        Creates a mask from the two sequences passed to be used in a sequence-pair classification task. An ALBERT
        sequence pair mask has the following format:

        ```
        0 0 0 0 0 0 0 0 0 0 0 1 1 1 1 1 1 1 1 1
        | first sequence    | second sequence |
        ```

        if token_ids_1 is None, only returns the first portion of the mask (0s).

        Note this is only used for back compatiblity, thus list of zero is returned.

        Args:
            token_ids_0 (`List[int]`):
                List of ids.
            token_ids_1 (`List[int]`, *optional*):
                Optional second list of IDs for sequence pairs.

        Returns:
            `List[int]`: List of zeros.
        """
        eos = [self.eos_token_id]

        if token_ids_1 is None:
            return len(token_ids_0 + eos) * [0]
        return len(token_ids_0 + eos + token_ids_1 + eos) * [0]
    

    @property
    def default_chat_template(self):
        return None


    def decode(
        self,
        token_ids,
        skip_special_tokens: bool = False,
        clean_up_tokenization_spaces: Optional[bool] = False,
        spaces_between_special_tokens: bool = False,
        **kwargs,
    ) -> str:
        # default spaces_between_special_tokens should be false.
        if spaces_between_special_tokens:
            logger.warning_once('spaces_between_special_tokens is set. \
                                It has no effect for bos,eos,pad,unk when transformers<=4.38.')
        return super().decode(
            token_ids,
            skip_special_tokens=skip_special_tokens,
            clean_up_tokenization_spaces=clean_up_tokenization_spaces,
            spaces_between_special_tokens=spaces_between_special_tokens,
            **kwargs,
        )