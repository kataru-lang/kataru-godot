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
const COMPILED_STORY_PATH = "res://kataru-story.yml"
const BOOKMARK_PATH = "user://kataru-bookmark.yml"
const DEFAULT_PASSAGE = "Start"
const DEBUG_LEVEL = 2

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
	print("Call next: ", input)
	self.ffi.next(input)


# Called when the node enters the scene tree for the first time.
func _ready():
	self.ffi.init(
		ProjectSettings.globalize_path(COMPILED_STORY_PATH),
		ProjectSettings.globalize_path(BOOKMARK_PATH),
		DEFAULT_PASSAGE,
		DEBUG_LEVEL
	)

	if !OS.has_feature("standalone"):
		self.ffi.compile(ProjectSettings.globalize_path(STORY_PATH))

	self.ffi.load()

	# Connect callbacks.
	self.ffi.dialogue.connect(
		func(char_name: String, text: String, attributes: String): self.dialogue.emit(
			char_name, text, JSON.parse_string(attributes)
		)
	)
	self.ffi.choices.connect(
		func(choice_list: Array, timeout: float): self.choices.emit(choice_list, timeout)
	)
	self.ffi.command.connect(
		func(cmd_name: String, params: String): self.command.emit(
			cmd_name, JSON.parse_string(params)
		)
	)
	self.ffi.input_command.connect(
		func(inputs: Dictionary, timeout: float): self.input.emit(inputs, timeout)
	)
	self.ffi.fatal.connect(func(message: String): assert(message != "", message))


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(_delta):
	pass
