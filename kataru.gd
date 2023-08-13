# Kataru - the simple YAML-based dialogue engine.
#
# This script is intended to be used as an Autoload:
# ```toml
# [autoload]
#
# Kataru="*res://kataru-godot/kataru.gd"
# ```
extends KataruInterface

const STORY_PATH = "res://kataru-story"
const BOOKMARK_PATH = "user://kataru-bookmark.yml"
const DEFAULT_PASSAGE = "Start"
const DEBUG_LEVEL = 0

signal dialogue(char_name: String, text: String, attributes: Array)
signal choices(choices: Array, timeout: float)
signal command(cmd_name: String, params: Dictionary)
signal input(input: Dictionary, timeout: float)


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
	self.input.emit(result["input"], result["timeout"])


# Called when the node enters the scene tree for the first time.
func _ready():
	# Connect callbacks.
	self.dialogue_json.connect(_on_dialogue_json)
	self.choices_json.connect(_on_choices_json)
	self.command_json.connect(_on_command_json)
	self.input_json.connect(_on_input_json)

	print(ProjectSettings.globalize_path(STORY_PATH))
	print(ProjectSettings.globalize_path(BOOKMARK_PATH))
	self.load(
		ProjectSettings.globalize_path(STORY_PATH),
		ProjectSettings.globalize_path(BOOKMARK_PATH),
		DEFAULT_PASSAGE,
		DEBUG_LEVEL
	)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(_delta):
	pass
