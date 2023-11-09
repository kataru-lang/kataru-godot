# Initialize dir structure
static func setup():
	# Setup root dir.
	if !DirAccess.dir_exists_absolute(Kataru.KATARU_DIR):
		DirAccess.make_dir_absolute(Kataru.KATARU_DIR)
		print("Kataru directory ", Kataru.KATARU_DIR, " was missing. Creating new directory...")
	# Set up story dir with a template.
	if !DirAccess.dir_exists_absolute(Kataru.STORY_PATH):
		print(
			"Kataru story directory ",
			Kataru.STORY_PATH,
			" was missing. Creating new story from template..."
		)
		DirAccess.make_dir_absolute(Kataru.STORY_PATH)
		var template_file = FileAccess.open(Kataru.TEMPLATE_PATH, FileAccess.READ)
		var template_str = template_file.get_as_text()
		var story_file = FileAccess.open(Kataru.STORY_PATH + "/main.yml", FileAccess.WRITE)
		story_file.store_string(template_str)

		var gdignore_file = FileAccess.open(Kataru.STORY_PATH + "/.gdignore", FileAccess.WRITE)
		gdignore_file.store_string("\n")
