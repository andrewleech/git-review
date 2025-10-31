use super::{App, CommentMode};
use crate::comments::{Comment, CommentLevel, CommitComments};

impl App {
    /// Load all comments for current branch
    pub fn load_comments(&mut self) {
        match crate::git::notes::read_all_for_branch(&self.repo, &self.current_branch) {
            Ok(all_comments) => {
                for commit_comments in all_comments {
                    self.comments_by_commit
                        .insert(commit_comments.commit_id.clone(), commit_comments);
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to load comments: {e}");
            }
        }
    }

    /// Get comments for currently selected commit
    pub fn current_commit_comments(&self) -> Option<&CommitComments> {
        self.selected_commit()
            .and_then(|c| self.comments_by_commit.get(&c.id.to_string()))
    }

    /// Start creating a comment based on cursor position
    pub fn start_comment_creation(&mut self) {
        if let Some(file) = self.selected_file() {
            // Detect comment context based on cursor position
            let (level, line_number, line_type, hunk_header) = self.detect_comment_context();

            self.comment_mode = CommentMode::Creating {
                level,
                file_path: file.new_path.clone(),
                line_number,
                line_type,
                hunk_header,
            };
            self.comment_draft.clear();
        } else {
            self.status_message = Some("No file selected".to_string());
        }
    }

    /// Save the current comment draft
    pub fn save_comment(&mut self) -> anyhow::Result<()> {
        if let CommentMode::Creating {
            level,
            file_path,
            line_number,
            line_type,
            hunk_header,
        } = &self.comment_mode
        {
            if self.comment_draft.trim().is_empty() {
                self.status_message = Some("Comment cannot be empty".to_string());
                return Ok(());
            }

            let comment = match level {
                CommentLevel::Line => Comment::new_line(
                    file_path.clone(),
                    line_number.ok_or_else(|| anyhow::anyhow!("Missing line number"))?,
                    line_type.ok_or_else(|| anyhow::anyhow!("Missing line type"))?,
                    self.comment_draft.clone(),
                )?,
                CommentLevel::Hunk => Comment::new_hunk(
                    file_path.clone(),
                    hunk_header
                        .as_ref()
                        .ok_or_else(|| anyhow::anyhow!("Missing hunk header"))?
                        .clone(),
                    self.comment_draft.clone(),
                )?,
                CommentLevel::File => {
                    Comment::new_file(file_path.clone(), self.comment_draft.clone())?
                }
            };

            if let Some(commit) = self.selected_commit() {
                let oid = commit.id;
                let cid = oid.to_string();
                let cc = self
                    .comments_by_commit
                    .entry(cid.clone())
                    .or_insert_with(|| {
                        CommitComments::new(cid.clone(), self.current_branch.clone())
                    });

                cc.add_comment(comment);
                crate::git::notes::write_comments(&self.repo, &self.current_branch, oid, cc)?;
                self.status_message = Some("Comment saved".to_string());
            }

            self.comment_mode = CommentMode::Normal;
            self.comment_draft.clear();
        }
        Ok(())
    }

    /// Cancel comment creation
    pub fn cancel_comment(&mut self) {
        self.comment_mode = CommentMode::Normal;
        self.comment_draft.clear();
    }

    /// View comments at current location (line, hunk, or file)
    pub fn view_comments_at_current_location(&mut self) {
        if let Some(file) = self.selected_file() {
            if let Some(commit_comments) = self.current_commit_comments() {
                let mut all_comments = Vec::new();

                // Detect what the cursor is pointing at
                let (level, line_number, line_type, hunk_header) = self.detect_comment_context();

                // Add line-level comments if cursor is on a line
                if level == CommentLevel::Line {
                    if let (Some(line_num), Some(l_type)) = (line_number, line_type) {
                        all_comments.extend(
                            commit_comments
                                .comments_at_line(&file.new_path, line_num, l_type)
                                .into_iter()
                                .cloned(),
                        );
                    }
                }

                // Add hunk-level comments if cursor is in a hunk
                if level == CommentLevel::Hunk || level == CommentLevel::Line {
                    if let Some(ref header) = hunk_header {
                        all_comments.extend(
                            commit_comments
                                .comments_at_hunk(&file.new_path, header)
                                .into_iter()
                                .cloned(),
                        );
                    }
                }

                // Always add file-level comments
                all_comments.extend(
                    commit_comments
                        .file_level_comments(&file.new_path)
                        .into_iter()
                        .cloned(),
                );

                if !all_comments.is_empty() {
                    self.comment_mode = CommentMode::ViewingComments(all_comments);
                } else {
                    self.status_message = Some("No comments at this location".to_string());
                }
            } else {
                self.status_message = Some("No comments for this commit".to_string());
            }
        }
    }

    /// Delete comment at index in view mode
    pub fn delete_comment_at_index(&mut self, index: usize) -> anyhow::Result<()> {
        let file = self.selected_file().map(|f| f.new_path.clone());
        let commit_oid = self.selected_commit().map(|c| c.id);

        if let (Some(file_path), Some(oid)) = (file, commit_oid) {
            let cid = oid.to_string();
            if let Some(cc) = self.comments_by_commit.get_mut(&cid) {
                let file_comment_indices: Vec<_> = cc
                    .comments
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.file_path == file_path)
                    .map(|(i, _)| i)
                    .collect();

                if let Some(&actual_idx) = file_comment_indices.get(index) {
                    cc.remove_comment(actual_idx);

                    if cc.is_empty() {
                        crate::git::notes::delete_comment(
                            &self.repo,
                            &self.current_branch,
                            oid,
                            0,
                        )?;
                        self.comments_by_commit.remove(&cid);
                    } else {
                        crate::git::notes::write_comments(
                            &self.repo,
                            &self.current_branch,
                            oid,
                            cc,
                        )?;
                    }

                    self.status_message = Some("Comment deleted".to_string());
                    self.comment_mode = CommentMode::Normal;
                }
            }
        }
        Ok(())
    }

    /// Close any open dialog/mode
    pub fn close_dialog(&mut self) {
        match self.comment_mode {
            CommentMode::Normal => {}
            CommentMode::Creating { .. } => {
                self.cancel_comment();
            }
            CommentMode::ViewingComments(_) => {
                self.comment_mode = CommentMode::Normal;
            }
        }
    }

    /// Clear status message
    #[allow(dead_code)]
    pub fn clear_status_message(&mut self) {
        self.status_message = None;
    }
}
