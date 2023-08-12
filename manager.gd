extends Kataru

const STORY_PATH = "res://kataru-story"
const BOOKMARK_PATH = "user://kataru-bookmark.yml"
const DEFAULT_PASSAGE = "Start"


# Called when the node enters the scene tree for the first time.
func _ready():
	print(ProjectSettings.globalize_path(STORY_PATH))
	print(ProjectSettings.globalize_path(BOOKMARK_PATH))
	self.load(
		ProjectSettings.globalize_path(STORY_PATH),
		ProjectSettings.globalize_path(BOOKMARK_PATH),
		DEFAULT_PASSAGE
	)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(_delta):
	pass
