from __future__ import annotations
from enum import Enum

# Version of the bitis compiler that generated this module ("{BITIS_VERSION}").
BITIS_VERSION: str
# Schema hash (0x940ea6464ca7ba60): first 8 bytes of SHA-256 over the canonical schema.
BITIS_SCHEMA_HASH: int



# Enums



# *** Enums for oneof



class OO_Bounded_Payload:
  @staticmethod
  def new_text(text: String) -> OO_Bounded_Payload: ...
  @staticmethod
  def new_raw(raw: String) -> OO_Bounded_Payload: ...
  @staticmethod
  def new_p(p: Point) -> OO_Bounded_Payload: ...



# *** Messages

class Point:
  def __init__(self, x: int, y: int,) -> None: ...

  @staticmethod
  def default() -> Point: ...

  def serialize(self) -> bytes:
    """
    Serialized the class instance to binary data

    :return: bytes representing the serialized class instance
    """
  @staticmethod
  def deserialize(data: bytes) -> Point:
    """
    Deserializes bytes to a python object

    :param data: binary data
    :return: An instance of the class with attributes set from the byte data
    """

  @property
  def x(self) -> int: ...
  @x.setter
  def x(self, v: int) -> None: ...
  @property
  def y(self) -> int: ...
  @y.setter
  def y(self, v: int) -> None: ...



class Bounded:
  """
  Bounded strings, bytes and lists
  """

  def __init__(self, name: String, blob: Vec<u8>, tags: list[String], note: String, points: list[Point], kinds: list[Kind], pair: list[Vec<u8>], unbounded: String, payload: OO_Bounded_Payload,) -> None: ...

  @staticmethod
  def default() -> Bounded: ...

  def serialize(self) -> bytes:
    """
    Serialized the class instance to binary data

    :return: bytes representing the serialized class instance
    """
  @staticmethod
  def deserialize(data: bytes) -> Bounded:
    """
    Deserializes bytes to a python object

    :param data: binary data
    :return: An instance of the class with attributes set from the byte data
    """

  @property
  def name(self) -> String: ...
  @name.setter
  def name(self, v: String) -> None: ...
  @property
  def blob(self) -> Vec<u8>: ...
  @blob.setter
  def blob(self, v: Vec<u8>) -> None: ...
  @property
  def tags(self) -> list[String]: ...
  @tags.setter
  def tags(self, v: list[String]) -> None: ...
  @property
  def note(self) -> String: ...
  @note.setter
  def note(self, v: String) -> None: ...
  @property
  def points(self) -> list[Point]: ...
  @points.setter
  def points(self, v: list[Point]) -> None: ...
  @property
  def kinds(self) -> list[Kind]: ...
  @kinds.setter
  def kinds(self, v: list[Kind]) -> None: ...
  @property
  def pair(self) -> list[Vec<u8>]: ...
  @pair.setter
  def pair(self, v: list[Vec<u8>]) -> None: ...
  @property
  def unbounded(self) -> String: ...
  @unbounded.setter
  def unbounded(self, v: String) -> None: ...
  @property
  def payload_oo(self) -> OO_Bounded_PayloadEnum: ...
  @property
  def payload_text(self) -> None | String: ...
  @payload_text.setter
  def payload_text(self, val: String) -> None: ...
  @property
  def payload_raw(self) -> None | String: ...
  @payload_raw.setter
  def payload_raw(self, val: String) -> None: ...
  @property
  def payload_p(self) -> None | Point: ...
  @payload_p.setter
  def payload_p(self, val: Point) -> None: ...



