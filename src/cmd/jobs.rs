use std::{fmt::Display, process::Child};

use anyhow::Result;

use crate::cmd::context::{Context, SharedState};

#[derive(PartialEq)]
pub enum JobStatus {
    Running,
    Done,
}

impl Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Running => write!(f, "Running                 "),
            JobStatus::Done => write!(f, "Done                    "),
        }
    }
    // fn into(self) -> String {
    //     match self {
    //         JobStatus::Running => format!("Running                 "),
    //         JobStatus::Done => format!("Done                    "),
    //     }
    // }
}

pub struct Job {
    pub id: usize,
    pub status: JobStatus,
    pub process: Child,
    pub cmd: String,
}

impl Job {
    pub fn new(id: usize, cmd: String, process: Child) -> Self {
        Self {
            id,
            status: JobStatus::Running,
            process,
            cmd,
        }
    }
    pub fn handle(ctx: &mut Context) -> Result<()> {
        let mut state = ctx.state.lock().unwrap();
        let mut symbols = vec!["-", "+"];
        let mut output_lines: Vec<String> = Vec::new();

        for job in state.jobs.iter_mut().rev() {
            match job.process.try_wait() {
                Ok(Some(_)) => {
                    job.status = {
                        job.cmd.pop();
                        JobStatus::Done
                    }
                }
                Ok(None) => job.status = JobStatus::Running,
                Err(e) => return Err(anyhow::anyhow!("Error checking wait status: {}", e)),
            }

            let sym = symbols.pop().unwrap_or(" ");
            output_lines.push(format!("[{}]{} {}{}", job.id, sym, job.status, job.cmd));
        }

        for line in output_lines.iter().rev() {
            ctx.stdout.writeln(line).unwrap();
        }

        state.jobs.retain(|job| job.status != JobStatus::Done);

        Ok(())
    }

    pub fn check_jobs(state: SharedState) -> Result<()> {
        let mut state = state.lock().unwrap();
        let mut symbols = vec!["-", "+"];
        let mut done_ids = vec![];

        for job in state.jobs.iter_mut().rev() {
            match job.process.try_wait() {
                Ok(Some(_)) => {
                    job.status = {
                        job.cmd.pop();
                        done_ids.push(job.id);
                        let sym = symbols.pop().unwrap_or(" ");
                        println!("[{}]{} {}{}", job.id, sym, JobStatus::Done, job.cmd);
                        JobStatus::Done
                    }
                }
                Ok(None) => job.status = JobStatus::Running,
                Err(e) => return Err(anyhow::anyhow!("Error checking wait status: {}", e)),
            }
        }

        state.jobs.retain(|job| job.status != JobStatus::Done);

        done_ids.iter().for_each(|id| {
            state.next_job_id.insert(*id);
        });

        Ok(())
    }
}
