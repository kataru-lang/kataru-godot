# Initialize dir structure
static func setup():
	# Setup root dir.
	if !DirAccess.dir_exists_absolute(Kataru.root_path):
		DirAccess.make_dir_absolute(Kataru.root_path)
		print("Kataru directory ", Kataru.root_path, " was missing. Creating new directory...")
	# Set up story dir with a template.
	if !DirAccess.dir_exists_absolute(Kataru.story_path):
		print(
			"Kataru story directory ",
			Kataru.story_path,
			" was missing. Creating new story from template..."
		)
		DirAccess.make_dir_absolute(Kataru.story_path)
		var template_file = FileAccess.open(Kataru.TEMPLATE_PATH, FileAccess.READ)
		var template_str = template_file.get_as_text()
		var story_file = FileAccess.open(Kataru.story_path + "/main.yml", FileAccess.WRITE)
		story_file.store_string(template_str)

		var gdignore_file = FileAccess.open(Kataru.story_path + "/.gdignore", FileAccess.WRITE)
		gdignore_file.store_string("\n")
