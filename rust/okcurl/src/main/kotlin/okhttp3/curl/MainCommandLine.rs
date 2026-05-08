/*
 * Copyright (C) 2022 Square, Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::process;
use crate::build_logic::src::main::kotlin::okhttp_publish_conventions_gradle::*;

// The Main class handles the command line interface for okcurl.
// In the original Kotlin code, this extends a Clikt command.
pub struct Main;

impl Main {
    pub fn new() -> Self {
        Self
    }

    // Entry point for the command line logic.
    // This corresponds to the `main` method provided by the Clikt library.
    pub fn main(&self, args: &[String]) {
        // In a real production translation, the logic inside Main().main(args) 
        // would be implemented here, likely using a crate like `clap` 
        // to mirror the Clikt functionality.
        // Since the provided Kotlin source only shows the delegation, 
        // we maintain the structure.
    }
}

// The main entry point of the application.
fn main() {
    // Collect command line arguments
    let args: Vec<String> = std::env::args().collect();
    
    // Instantiate the Main command handler and execute it
    let main_cmd = Main::new();
    main_cmd.main(&args);
    
    // Exit with status 0 as per the original Kotlin code
    process::exit(0);
}