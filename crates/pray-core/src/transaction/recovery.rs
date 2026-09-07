use super::*;

impl Journal {
    pub fn recover(&mut self) -> PrayResult<()> {
        self.clean_stages()?;
        if !self.path.try_exists()? {
            return Ok(());
        }
        let mut file =
            crate::render_file::open_regular(&self.path, "project recovery journal", true)?;
        if file.metadata()?.len() > MAX_LOG {
            return Err(failure("project recovery journal exceeds its 96 MiB limit"));
        }
        let mut bytes = Vec::new();
        (&mut file).take(MAX_LOG + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_LOG {
            return Err(failure("project recovery journal exceeds its 96 MiB limit"));
        }
        let complete = bytes
            .iter()
            .rposition(|byte| *byte == b'\n')
            .map_or(0, |index| index + 1);
        let mut entries = Vec::new();
        let mut undone = BTreeSet::new();
        let mut committed = false;
        for (index, line) in bytes[..complete]
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .enumerate()
        {
            let record: Record = serde_json::from_slice(line)
                .map_err(|_| failure("invalid project recovery journal"))?;
            match record {
                Record::Start { version: 1 } if index == 0 => {}
                Record::Write { entry } if index > 0 && !committed && undone.is_empty() => {
                    if entries.len() == MAX_ENTRIES {
                        return Err(failure("too many recovery entries"));
                    }
                    self.destination(&entry.path)?;
                    if entry.mode > 0o777 {
                        return Err(failure("invalid recovery permissions"));
                    }
                    entries.push(entry);
                }
                Record::Undone { index } if !committed && index < entries.len() => {
                    undone.insert(index);
                }
                Record::Commit if index > 0 && undone.is_empty() && !committed => {
                    committed = true;
                }
                _ => return Err(failure("invalid project recovery journal state")),
            }
        }
        file.set_len(complete as u64)?;
        file.sync_all()?;
        if !committed {
            for (index, entry) in entries.iter().enumerate().rev() {
                if undone.contains(&index) {
                    continue;
                }
                let before = entry
                    .before
                    .as_ref()
                    .map(|text| STANDARD.decode(text))
                    .transpose()
                    .map_err(|_| failure("invalid recovery bytes"))?;
                if before
                    .as_ref()
                    .is_some_and(|bytes| bytes.len() > 32 * 1024 * 1024)
                {
                    return Err(failure("recovery file exceeds 32 MiB"));
                }
                let path = self.destination(&entry.path)?;
                let current = snapshot(&path)?;
                if current != before {
                    if current.is_some()
                        && current.as_deref().map(sha256_prefixed) != entry.after_hash
                    {
                        return Err(changed(&entry.path));
                    }
                    self.install(
                        &entry.path,
                        current.as_deref(),
                        before.as_deref(),
                        entry.mode,
                    )?;
                }
                // A prior recovery may have stopped after replacement but before directory sync.
                self.sync_parents(&path)?;
                self.append(&Record::Undone { index })?;
            }
        }
        fs::remove_file(&self.path)?;
        sync_directory(&self.directory)?;
        self.saved = 0;
        self.entries = 0;
        self.clean_stages()
    }
}
