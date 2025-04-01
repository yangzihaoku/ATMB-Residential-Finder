use std::io::{self, Write};
use log::info;
use crate::atmb::ATMBCrawl;
use crate::atmb::model::Mailbox;

pub struct CliApp {
    atmb: ATMBCrawl,
}

impl CliApp {
    pub fn new() -> color_eyre::Result<Self> {
        Ok(Self {
            atmb: ATMBCrawl::new()?,
        })
    }

    pub async fn run(&self) -> color_eyre::Result<Vec<Mailbox>> {
        // 获取所有可用的州
        let states = self.atmb.get_available_states().await?;
        
        // 显示州列表并让用户选择
        let selected_states = self.select_states(&states)?;
        
        if selected_states.is_empty() {
            info!("No states selected, fetching all states...");
            self.atmb.fetch().await
        } else {
            info!("Fetching selected states: {:?}", selected_states);
            self.atmb.fetch_selected_states(&selected_states).await
        }
    }

    fn select_states(&self, states: &[String]) -> color_eyre::Result<Vec<String>> {
        println!("Available states:");
        for (i, state) in states.iter().enumerate() {
            println!("{}: {}", i + 1, state);
        }
        println!("\nEnter the numbers of states you want to fetch (comma separated), or press Enter to fetch all states:");
        
        let mut input = String::new();
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;
        
        let input = input.trim();
        if input.is_empty() {
            return Ok(vec![]);
        }
        
        let selected: Vec<String> = input
            .split(',')
            .filter_map(|s| s.trim().parse::<usize>().ok())
            .filter(|&i| i > 0 && i <= states.len())
            .map(|i| states[i - 1].clone())
            .collect();
        
        Ok(selected)
    }
}