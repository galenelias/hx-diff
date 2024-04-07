use std::process::Command;

#[derive(Debug)]
pub struct GitError {}

#[derive(Debug)]
pub struct GitStatus {
	pub branch_oid: String,
	pub branch_head: String,
	pub branch_upstrem: String,
	pub entries: Vec<Entry>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EntryStatus {
	Added,
	Untracked,
	Modified,
	Deleted,
	Renamed,
	None,
}

impl EntryStatus {
	pub fn from_u8(status: &u8) -> EntryStatus {
		let status_char = *status as char;
		match status_char {
			'A' => EntryStatus::Added,
			'M' => EntryStatus::Modified,
			'D' => EntryStatus::Deleted,
			'.' => EntryStatus::None,
			_ => panic!("Unknown status: {}", status_char),
		}
	}

	pub fn to_string(&self) -> String {
		match self {
			EntryStatus::Added => "Added".to_string(),
			EntryStatus::Untracked => "Untracked".to_string(),
			EntryStatus::Modified => "Modified".to_string(),
			EntryStatus::Deleted => "Deleted".to_string(),
			EntryStatus::Renamed => "Renamed".to_string(),
			EntryStatus::None => "None".to_string(),
		}
	}
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct Sha1Hash([u8; 40]);

#[derive(Debug)]
pub struct Entry {
	pub staged_status: EntryStatus,
	pub unstaged_status: EntryStatus,
	// TODO: Add more fields
	pub head_sha1: Sha1Hash,
	pub index_sha1: Sha1Hash,
	pub path: std::path::PathBuf,
	pub original_path: Option<String>,
}

pub fn get_status(/*TODO: More options? */) -> Result<GitStatus, GitError> {
	let output = Command::new("git")
		.arg("status")
		.arg("--ignore-submodules=all")
		.arg("--branch")
		.arg("--porcelain=v2")
		.arg("--")
		.output()
		.expect("failed to execute process");

	let output_string = String::from_utf8(output.stdout).expect("Invalid utf-8");

	parse_status(&output_string)
}

fn parse_status(status: &str) -> Result<GitStatus, GitError> {
	let mut branch_oid = String::new();
	let mut branch_head = String::new();
	let mut branch_upstrem = String::new();

	let mut entries = Vec::new();

	for line in status.lines() {
		if line.starts_with("# branch.oid ") {
			branch_oid = line[13..].to_string();
		} else if line.starts_with("# branch.head ") {
			branch_head = line[14..].to_string();
		} else if line.starts_with("# branch.upstream ") {
			branch_upstrem = line[18..].to_string();
		} else if line.starts_with("1 ") {
			// 1 A. N... 000000 100644 100644 0000000000000000000000000000000000000000 ea8c4bf7f35f6f77f75d92ad8ce8349f6e81ddba .gitignore
			let mut iter = line.split_whitespace().skip(1);
			let file_status = iter.next().unwrap();
			let staged_status = EntryStatus::from_u8(&file_status.as_bytes()[0]);
			let unstaged_status = EntryStatus::from_u8(&file_status.as_bytes()[1]);

			iter.nth(3); // Skip: file mode for HEAD, index, worktree
			 // TODO: Handle rename confidence parameter

			let head_sha1 = iter.next().unwrap().to_owned();
			let index_sha1 = iter.next().unwrap().to_owned();

			let path = iter.next().unwrap();

			entries.push(Entry {
				staged_status,
				unstaged_status,
				head_sha1: Sha1Hash(head_sha1.as_bytes().try_into().unwrap()),
				index_sha1: Sha1Hash(index_sha1.as_bytes().try_into().unwrap()),
				path: std::path::Path::new(path).canonicalize().unwrap(),
				original_path: None,
			});
		}
	}

	Ok(GitStatus {
		branch_oid,
		branch_head,
		branch_upstrem,
		entries,
	})
}

pub fn get_diff(path: &std::path::Path, is_staged: bool) -> Result<String, GitError> {
	let mut cmd = Command::new("git");
	cmd.arg("diff").arg("-p");

	if is_staged {
		cmd.arg("--staged");
	}

	let cmd = cmd.arg("--").arg(path);

	let output = cmd.output().expect("failed to execute process");

	if !output.status.success() {
		println!("Error: Failed to run git diff");
		println!("---");
		println!(
			"{}",
			String::from_utf8(output.stderr).expect("Invalid utf-8")
		);
		return Err(GitError {});
	}

	let output_string = String::from_utf8(output.stdout).expect("Invalid utf-8");

	if let Some(body_start) = output_string.find("@@ ") {
		Ok(output_string[body_start..].to_string())
	} else {
		Err(GitError {})
	}
}

pub fn get_file_contents(path: &std::path::Path, sha1: &Sha1Hash) -> Result<String, GitError> {
	let output = Command::new("git")
		.arg("cat-file")
		.arg("--filters")
		.arg(format!("--path={}", path.display()))
		.arg(std::str::from_utf8(&sha1.0).unwrap())
		.output()
		.expect("failed to execute process");

	if !output.status.success() {
		println!("Error: Failed to run git cat-file");
		println!("---");
		println!(
			"{}",
			String::from_utf8(output.stderr).expect("Invalid utf-8")
		);
		return Err(GitError {});
	}

	Ok(String::from_utf8(output.stdout).expect("Invalid utf-8"))
}
