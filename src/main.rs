use console::Color;
use reqwest::{
    self,
    header::{AUTHORIZATION, USER_AGENT},
};
use serde::{de::Error, Deserialize, Serialize};
use serde_json::json;
use std::{
    env,
    fs::{self, File},
    io::{self, BufRead, BufReader, Read, Write},
    path::Path,
};
use termion::{
    color::{self, Reset},
    cursor::{self, DetectCursorPos},
    input::TermRead,
    raw::IntoRawMode,
    style,
};

mod resp_structs;
use resp_structs::*;

async fn get_user_info(login: &str) {
    let mut url = String::from("https://api.github.com/users/");
    url += &login;

    let client = reqwest::Client::new();
    let mut response = client
        .get(url)
        .header(USER_AGENT, "ghfetch")
        .send()
        .await
        .unwrap();
    if response.status().is_success() {
        let strData = &response.text().await.unwrap();
        let userData: UserData = serde_json::from_str(strData).unwrap();

        println!();

        print_logo();
        println!(
            "User: {} {} ({}) {} ",
            color::Fg(color::Red),
            userData.login,
            userData.name,
            color::Fg(color::Reset)
        );
        //println!(r#"Bio: '{}'"#, userData.bio);
    }
}

static mut logo_index: usize = 0;

const gh_logo_vec: [&str; 33] = [
    "                          @@@@@@@@@                          ",
    "                   @@@@@@@@@@@@@@@@@@@@@@@                   ",
    "               @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@               ",
    "             @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@             ",
    "          @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@          ",
    "        @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@        ",
    "       @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@       ",
    "     @@@@@@@@@   @@@@@@@@@@@@@@@@@@@@@@@@@@    @@@@@@@@@     ",
    "    @@@@@@@@@@       @@@@@@@     @@@@@@@       @@@@@@@@@@    ",
    "   @@@@@@@@@@@                                 @@@@@@@@@@@   ",
    "  @@@@@@@@@@@@                                 @@@@@@@@@@@@  ",
    "  @@@@@@@@@@@@                                 @@@@@@@@@@@@  ",
    " @@@@@@@@@@@@                                   @@@@@@@@@@@@ ",
    " @@@@@@@@@@@                                     @@@@@@@@@@@ ",
    "@@@@@@@@@@@                                       @@@@@@@@@@@",
    "@@@@@@@@@@@                                       @@@@@@@@@@@",
    "@@@@@@@@@@@                                       @@@@@@@@@@@",
    "@@@@@@@@@@@                                       @@@@@@@@@@@",
    "@@@@@@@@@@@@                                     @@@@@@@@@@@@",
    "@@@@@@@@@@@@                                     @@@@@@@@@@@@",
    " @@@@@@@@@@@@                                   @@@@@@@@@@@@ ",
    " @@@@@@@@@@@@@                                 @@@@@@@@@@@@@ ",
    "  @@@@@@@@@@@@@@                             @@@@@@@@@@@@@@  ",
    "  @@@@@@   @@@@@@@@                       @@@@@@@@@@@@@@@@@  ",
    "   @@@@@@@   @@@@@@@@@@@@           @@@@@@@@@@@@@@@@@@@@@@   ",
    "    @@@@@@@@   @@@@@@@@@             @@@@@@@@@@@@@@@@@@@@    ",
    "      @@@@@@@    @@@@@@               @@@@@@@@@@@@@@@@@      ",
    "       @@@@@@                         @@@@@@@@@@@@@@@@       ",
    "         @@@@@@@                      @@@@@@@@@@@@@@         ",
    "           @@@@@@@@@@@@               @@@@@@@@@@@@           ",
    "             @@@@@@@@@@               @@@@@@@@@@             ",
    "                @@@@@@@               @@@@@@@                ",
    "                    @@                 @@                    ",
];

fn print_logo() {
    unsafe {
        print!("{}      ", gh_logo_vec[logo_index]);
        logo_index += 1;
    }
}

async fn get_user_work_info(login: &str, token: &str) {
    let url = "https://api.github.com/graphql".to_string();
    let json_body = r#"
        query {
            user(login: ""#
        .to_string()
        + login
        + r#"") {
                contributionsCollection{
                    contributionCalendar{
                        totalContributions,
                        weeks{
                            contributionDays{
                                contributionCount
                                date
                            }
                        }
                        
                    }
                }
                pinnedItems(first: 6, types: REPOSITORY) {
                    nodes {
                        ... on Repository {
                            name
                            description
                            forks{
                                totalCount
                            }
                            stargazers{
                                totalCount     
                            }
                        }
                    }
                }
            }
        }
    "#;

    let graph = GraphQLRequest {
        query: json_body.to_string(),
    };

    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .header(USER_AGENT, "ghfetch")
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&graph)
        .send()
        .await
        .unwrap();

    let graph_data = response.text().await.unwrap();

    let graph_resp_data: GraphRespData = serde_json::from_str(&graph_data).unwrap();

    print_logo();
    println!(
        "Total contributions {}",
        graph_resp_data
            .data
            .user
            .contributions_collection
            .contribution_calendar
            .total_contributions
    );
    print_logo();
    println!("Pinned Repos:");
    for node in graph_resp_data.data.user.pinned_items.nodes {
        print_logo();
        println!("Repo: {} ", node.name);
        print_logo();
        println!("{}", node.description);
        print_logo();
        println!(
            r#" * {} \|/ {}"#,
            node.stargazers.total_count, node.forks.total_count
        );
    }

    for i in 0..7 {
        print_logo();
        for week in graph_resp_data
            .data
            .user
            .contributions_collection
            .contribution_calendar
            .weeks
            .iter()
        {
            if i < week.contribution_days.len() {
                //print!("{}", week.contribution_days[i].contribution_count);
                print_activity_square(week.contribution_days[i].contribution_count);
            }
        }
        println!();
    }
}

fn print_activity_square(contribution_count: u32) {
    if contribution_count == 0 {
        print!(
            "{}#{}",
            color::Fg(color::Rgb(47, 52, 59)),
            color::Fg(color::Reset)
        );
        return;
    }
    if contribution_count < 4 {
        print!(
            "{}#{}",
            color::Fg(color::Rgb(14, 68, 41)),
            color::Fg(color::Reset)
        );
        return;
    }
    if contribution_count < 8 {
        print!(
            "{}#{}",
            color::Fg(color::Rgb(0, 109, 50)),
            color::Fg(color::Reset)
        );
        return;
    }
    if contribution_count < 10 {
        print!(
            "{}#{}",
            color::Fg(color::Rgb(38, 166, 65)),
            color::Fg(color::Reset)
        );
        return;
    } else {
        print!(
            "{}#{}",
            color::Fg(color::Rgb(57, 211, 83)),
            color::Fg(color::Reset)
        );
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if is_config_file_exists() {
        let (login, token) = read_config_file().unwrap();

        get_user_info(&login).await;
        get_user_work_info(&login, &token).await;
    } else {
        match args.len() {
            1 => panic!("Error: Enter GitHub username + token"),
            2 => {
                get_user_info(&args[1]).await;
                println!("Enter GitHub token for full information")
            }
            3 => {
                println!("Save arguments to the config file?");
                println!("Yes = [Y] No = [N]");
                let mut read_buf = [0u8; 1];
                io::stdin()
                    .read_exact(&mut read_buf)
                    .expect("Couldn't read the input character");

                let ch = read_buf[0] as char;
                let ch = ch.to_lowercase().next().unwrap();
                if ch == 'y' {
                    create_config_file(&args[1], &args[2])
                }

                get_user_info(&args[1]).await;
                get_user_work_info(&args[1], &args[2]).await;
            }
            _ => {}
        }
    }

    // Starting from index 1 because the first argument is the binary location

    unsafe {
        while logo_index != gh_logo_vec.len() {
            print_logo();
            println!();
        }
    }
}

//fn parse_args(args: Vec<String>) {
//    match args.len() {
//        1 => panic!("Error: Enter GitHub username and Token"),
//        2 => panic!("Error: Enter  GitHub Token"),
//        _ => {}
//    }
//
//    let login = args[1].clone();
//    let token = args[2].clone();
//    create_config_file(login, token);
//}

fn is_config_file_exists() -> bool {
    let file_path = Path::new("config");
    file_path.exists()
}
fn read_config_file() -> io::Result<(String, String)> {
    let mut login = String::new();
    let mut token = String::new();
    let file = File::open("config").expect("Couldn't open the config file");
    let reader = BufReader::new(file);
    for (i, line_content) in reader.lines().enumerate() {
        match i {
            0 => login = line_content?,
            1 => token = line_content?,
            _ => {}
        }
    }
    Ok((login, token))
}

fn create_config_file(login: &str, token: &str) {
    let mut file = File::create("config").expect("Couldn't create config file");
    let _ = file.write(login.as_bytes());
    let _ = file.write(b"\n");
    let _ = file.write(token.as_bytes());
}
