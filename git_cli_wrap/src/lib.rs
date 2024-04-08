use std::process::Command;

#[derive(Debug)]
pub struct GitError {}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct Sha1Hash([u8; 40]);

impl Sha1Hash {
	pub fn from_bytes(bytes: &[u8]) -> Sha1Hash {
		let mut hash = [0; 40];
		hash.copy_from_slice(&bytes);
		Sha1Hash(hash)
	}
}

#[derive(Debug)]
pub struct StatusEntry {
	pub staged_status: EntryStatus,
	pub unstaged_status: EntryStatus,
	// TODO: Add more fields
	pub head_sha1: Sha1Hash,
	pub index_sha1: Sha1Hash,
	pub path: std::path::PathBuf,
}

#[derive(Debug)]
pub struct GitStatus {
	pub branch_oid: String,
	pub branch_head: String,
	pub branch_upstream: String,
	pub entries: Vec<StatusEntry>,
}

#[derive(Debug)]
pub struct ShowEntry {
	pub left_status: EntryStatus,
	pub right_status: EntryStatus,
	pub left_sha1: Sha1Hash,
	pub right_sha1: Sha1Hash,
	pub path: std::path::PathBuf,
}

#[derive(Debug)]
pub struct GitShow {
	pub description: String,
	pub entries: Vec<ShowEntry>,
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
	let mut branch_upstream = String::new();

	let mut entries = Vec::new();

	for line in status.lines() {
		if line.starts_with("# branch.oid ") {
			branch_oid = line[13..].to_string();
		} else if line.starts_with("# branch.head ") {
			branch_head = line[14..].to_string();
		} else if line.starts_with("# branch.upstream ") {
			branch_upstream = line[18..].to_string();
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

			entries.push(StatusEntry {
				staged_status,
				unstaged_status,
				head_sha1: Sha1Hash(head_sha1.as_bytes().try_into().unwrap()),
				index_sha1: Sha1Hash(index_sha1.as_bytes().try_into().unwrap()),
				path: std::path::Path::new(path).canonicalize().unwrap(),
			});
		}
	}

	Ok(GitStatus {
		branch_oid,
		branch_head,
		branch_upstream,
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
	// Null/Empty file case
	if sha1 == &Sha1Hash(['0' as u8; 40]) {
		return Ok("".to_string());
	}

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

pub fn show(commit: &str) -> Result<GitShow, GitError> {
	let output = Command::new("git")
		.arg("show")
		.arg("--abbrev=40")
		.arg("--raw")
		.arg(commit)
		.output()
		.expect("failed to execute process");

	if !output.status.success() {
		println!("Error: Failed to run git show");
		println!("---");
		println!(
			"{}",
			String::from_utf8(output.stderr).expect("Invalid utf-8")
		);
		return Err(GitError {});
	}

	let output_string = String::from_utf8(output.stdout).expect("Invalid utf-8");

	let mut description = String::new();
	let mut entries = Vec::new();

	for line in output_string.lines() {
		if !line.starts_with(':') {
			description.push_str(line);
		} else {
			let mut iter = line[1..].split_whitespace();
			let _mode1 = iter.next().unwrap();
			let _mode2 = iter.next().unwrap();
			let left_sha1 = iter.next().unwrap();
			let right_sha1 = iter.next().unwrap();
			let _mode = iter.next().unwrap();
			let path = iter.next().unwrap();

			entries.push(ShowEntry {
				left_status: EntryStatus::None,
				right_status: EntryStatus::None,
				left_sha1: Sha1Hash::from_bytes(left_sha1.as_bytes()),
				right_sha1: Sha1Hash::from_bytes(right_sha1.as_bytes()),
				path: std::path::Path::new(path).canonicalize().unwrap(),
			});
		}
	}

	Ok(GitShow {
		description,
		entries,
	})
}
