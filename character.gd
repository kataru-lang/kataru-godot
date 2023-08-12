extends Node


# Called when the node enters the scene tree for the first time.
func _ready():
	KataruManager.dialogue.connect(_on_dialogue)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(_delta):
	pass


func _on_dialogue(name: String, text: String):
	if self.name == name:
		print(name, ": ", text)
