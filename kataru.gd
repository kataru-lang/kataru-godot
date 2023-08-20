# Kataru - the simple YAML-based dialogue engine.
#
# This script is intended to be used as an Autoload:
# ```toml
# [autoload]
#
# Kataru="*res://kataru-godot/kataru.gd"
# ```
@tool
extends Node

const STORY_PATH = "res://kataru-story"
const COMPILED_STORY_PATH = "res://kataru-story.bin"
const BOOKMARK_PATH = "user://kataru-bookmark.yml"
const DEFAULT_PASSAGE = "Start"
const DEBUG_LEVEL = 1

# Interface with Rust.
var ffi = KataruInterface.new()

signal dialogue(char_name: String, text: String, attributes: Array)
signal choices(choices: Array, timeout: float)
signal command(cmd_name: String, params: Dictionary)
signal input_command(input: Dictionary, timeout: float)


func run_passage(passage: String):
	self.ffi.run_passage(passage)

func run_passage_until_choice(passage: String):
	self.ffi.run_passage_until_choice(passage)

func next(input: String):
	self.ffi.next(input)

func _on_dialogue_json(json: String):
	var result = JSON.parse_string(json)
	if result == null:
		return null
	self.dialogue.emit(result["name"], result["text"], result["attributes"])


func _on_choices_json(json: String):
	var result = JSON.parse_string(json)
	if result == null:
		return null
	self.choices.emit(result["choices"], result["timeout"])


func _on_command_json(json: String):
	var result = JSON.parse_string(json)
	if result == null:
		return null
	self.command.emit(result["name"], result["params"])


func _on_input_json(json: String):
	var result = JSON.parse_string(json)
	if result == null:
		return null
	self.input_command.emit(result["input"], result["timeout"])

# Called when the node enters the scene tree for the first time.
func _ready():
	self.ffi.init(
		ProjectSettings.globalize_path(COMPILED_STORY_PATH),
		ProjectSettings.globalize_path(BOOKMARK_PATH),
		DEFAULT_PASSAGE,
		DEBUG_LEVEL)

	if !OS.has_feature("standalone"):
		self.ffi.compile(ProjectSettings.globalize_path(STORY_PATH))
	
	self.ffi.load()

	# Connect callbacks.
	self.ffi.dialogue_json.connect(_on_dialogue_json)
	self.ffi.choices_json.connect(_on_choices_json)
	self.ffi.command_json.connect(_on_command_json)
	self.ffi.input_json.connect(_on_input_json)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(_delta):
	pass
