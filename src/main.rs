use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use rand::Rng;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

fn main() {
    // Create a new Bevy app and add default plugins and the EguiPlugin for UI
    App::new()
        .add_plugins(DefaultPlugins)  // Adds default plugins (audio, window, etc.)
        .add_plugins(EguiPlugin)  // Adds Egui plugin for UI functionality
        .insert_resource(ClearColor(Color::rgb(214.0 / 255.0, 204.0 / 255.0, 185.0 / 255.0))) // Set the background color of the window
        .add_systems(Update, ui_system) // Register the UI update system
        .run(); // Run the application
}

fn ui_system(
    mut contexts: EguiContexts,  // Access the Egui context for UI updates
    mut input_text: Local<String>,  // Holds the text content for file operations
    mut files_and_folders: Local<Vec<PathBuf>>,  // Holds files and folders in the current directory
    mut loaded_file: Local<Option<PathBuf>>,  // Holds the path of the currently loaded file
    mut current_dir_str: Local<String>,  // Holds the current directory as a string
    mut show_file_popup: Local<bool>,  // Flag to show the file creation popup
    mut show_folder_popup: Local<bool>,  // Flag to show the folder creation popup
    mut show_save_popup: Local<bool>,  // Flag to show the save popup for file content
) {
    let ctx = contexts.ctx_mut();  // Get mutable reference to the Egui context

    // Initialize image loaders for Egui (if needed)
    egui_extras::install_image_loaders(ctx);

    // If the current directory string is empty, set it to "./root"
    if current_dir_str.is_empty() {
        *current_dir_str = String::from("./root");
    }

    let dir_path = Path::new(current_dir_str.as_str());  // Convert current directory string to Path

    // Read the directory contents if the directory exists
    if dir_path.exists() {
        let entries = match fs::read_dir(dir_path) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("Error reading directory: {}", e);
                return;
            }
        };

        let mut file_paths = Vec::new();  // Vector to store file paths
        let mut folder_paths = Vec::new();  // Vector to store folder paths

        // Loop through the entries in the directory and categorize them into files and folders
        for entry in entries.filter_map(|entry| entry.ok()) {
            let path = entry.path();
            if path.is_file() {
                file_paths.push(path);
            } else if path.is_dir() {
                folder_paths.push(path);
            }
        }

        // Concatenate the folders and files into one list
        *files_and_folders = [folder_paths, file_paths].concat();
    }

    // Central panel to show the main UI
    egui::CentralPanel::default()
        .frame(egui::Frame::default().inner_margin(egui::vec2(20.0, 20.0))) // Add inner margin
        .show(ctx, |ui| {
            ui.vertical(|ui| {  // Layout UI vertically
                // Show the current directory input field
                ui.horizontal(|ui| {
                    ui.label("Current Directory: ");
                    ui.text_edit_singleline(&mut *current_dir_str);
                });

                // Button to create a new file
                if ui.button("Create File").clicked() {
                    *show_file_popup = true;  // Set the file creation popup flag to true
                    *input_text = String::new(); // Clear any previous input text
                }

                // Button to create a new folder
                if ui.button("Create Folder").clicked() {
                    *show_folder_popup = true;  // Set the folder creation popup flag to true
                }

                // Show the file creation popup
                if *show_file_popup {
                    egui::Window::new("Create a New File")
                        .resizable(false)
                        .show(ctx, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Enter content for new file...");
                            });

                            // Text editor for entering file content
                            ui.add_sized(
                                egui::vec2(ui.available_width(), ui.available_height() - 40.0),
                                egui::TextEdit::multiline(&mut *input_text).desired_rows(10),
                            );

                            // Buttons to save or cancel the file creation
                            ui.horizontal(|ui| {
                                if ui.button("Save").clicked() {
                                    let random_file_name = format!("file_{}.txt", generate_random_number());  // Generate a random file name
                                    create_file(&random_file_name, &*current_dir_str, &*input_text);  // Create the file and save content
                                    *show_file_popup = false;  // Close the file popup
                                }
                                if ui.button("Cancel").clicked() {
                                    *show_file_popup = false;  // Close the file popup
                                }
                            });
                        });
                }

                // Show the folder creation popup
                if *show_folder_popup {
                    let random_folder_name = format!("folder_{}", generate_random_number());  // Generate a random folder name
                    create_folder(&random_folder_name, &*current_dir_str);  // Create the folder
                    *show_folder_popup = false;  // Close the folder popup
                }

                // Display files and folders in the current directory
                ui.horizontal(|ui| {
                    ui.label("Files and Folders in current directory: ");
                    for item in files_and_folders.iter() {
                        let item_name = item.file_name().unwrap_or_default().to_string_lossy();

                        // Handle directory item
                        if item.is_dir() {
                            ui.vertical(|ui| {
                                let logo = ui.add(egui::ImageButton::new(
                                    egui::Image::new(egui::include_image!("assets/folder.png"))
                                        .fit_to_exact_size(egui::vec2(75.0, 75.0)),
                                ));

                                // Directory clicked: Update the current directory path
                                if logo.clicked() {
                                    *current_dir_str = format!("{}/{}", *current_dir_str, item_name);
                                    println!("Updated current directory to: {}", *current_dir_str);
                                }

                                // Handle right-click on the folder
                                if logo.secondary_clicked() {
                                    println!("Right-clicked on folder: {}", item_name);
                                }

                                // Folder context menu
                                logo.context_menu(|ui| {
                                    if ui.button("Close the menu").clicked() {
                                        ui.close_menu();
                                    }
                                });

                                ui.label(item_name);  // Show the folder name
                            });
                        } else {  // Handle file item
                            ui.vertical(|ui| {
                                let logo = ui.add(egui::ImageButton::new(
                                    egui::Image::new(egui::include_image!("assets/file.png"))
                                        .fit_to_exact_size(egui::vec2(75.0, 75.0)),
                                ).frame(true));

                                // File clicked: Open the file content for editing
                                if logo.clicked() {
                                    open_file_content(item, &mut *input_text, &mut *loaded_file);
                                    *show_save_popup = true;
                                }

                                // Handle right-click on the file
                                if logo.secondary_clicked() {
                                    println!("Right-clicked on file: {}", item_name);
                                }

                                // File context menu
                                logo.context_menu(|ui| {
                                    if ui.button("Close the menu").clicked() {
                                        ui.close_menu();
                                    }
                                });

                                ui.label(item_name);  // Show the file name
                            });
                        }
                    }
                });

                // Display the currently loaded file if any
                if let Some(ref path) = *loaded_file {
                    ui.label(format!(
                        "Loaded File: {}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    ));
                } else {
                    ui.label("No file loaded.");
                }

                // Add space for better UI layout
                let top_half_height = ui.available_height() / 2.0;
                ui.add_space(top_half_height);

                // Show the save popup if a file is loaded
                if *show_save_popup {
                    egui::Window::new("Save/Cancel")
                        .resizable(false)
                        .show(ctx, |ui| {
                            ui.horizontal(|ui| {
                                if ui.button("Save").clicked() {
                                    if let Some(ref file_path) = *loaded_file {
                                        save_content(&*input_text, file_path.as_path());  // Save content to the file
                                    }
                                    *show_save_popup = false;  // Close the save popup
                                }
                                if ui.button("Cancel").clicked() {
                                    *input_text = String::new();  // Clear input text
                                    *loaded_file = None;  // Reset loaded file
                                    *show_save_popup = false;  // Close the save popup
                                }
                            });

                            // Text editor to modify file content
                            ui.add_sized(
                                egui::vec2(ui.available_width(), ui.available_height() - 40.0),
                                egui::TextEdit::multiline(&mut *input_text).desired_rows(10),
                            );
                        });
                }
            });
        });
}

// Generates a random number to append to file/folder names
fn generate_random_number() -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(10000..99999)
}

// Creates a new file with the specified name, writes content to it
fn create_file(file_name: &str, current_dir_str: &str, content: &str) {
    let full_path = Path::new(current_dir_str).join(file_name);  // Full file path
    if let Err(e) = File::create(&full_path) {  // Create the file
        eprintln!("Error creating file: {}", e);
    } else {
        println!("File created: {:?}", full_path);
        save_content(content, &full_path);  // Save the provided content to the file
    }
}

// Creates a new folder with the specified name
fn create_folder(folder_name: &str, current_dir_str: &str) {
    let folder_path = Path::new(current_dir_str).join(folder_name);  // Full folder path
    if let Err(e) = fs::create_dir(&folder_path) {  // Create the folder
        eprintln!("Error creating folder: {}", e);
    } else {
        println!("Folder created: {:?}", folder_path);
    }
}

// Saves the input text to the specified file path
fn save_content(input_text: &str, file_path: &Path) {
    let mut file = match File::create(file_path) {  // Open the file for writing
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening file for writing: {}", e);
            return;
        }
    };

    if let Err(e) = file.write_all(input_text.as_bytes()) {  // Write content to file
        eprintln!("Error writing to file: {}", e);
    } else {
        println!("Content saved to {:?}", file_path);
    }
}

// Opens the content of a file and loads it into the input_text editor
fn open_file_content(file_path: &Path, input_text: &mut String, loaded_file: &mut Option<PathBuf>) {
    let mut file = File::open(file_path).unwrap_or_else(|_| {  // Open the file for reading
        eprintln!("Error opening file: {:?}", file_path);
        std::process::exit(1);
    });

    let mut content = String::new();  // String to hold the file content
    if let Err(e) = file.read_to_string(&mut content) {  // Read the file content
        eprintln!("Error reading file: {:?}", e);
    } else {
        *input_text = content;  // Load the content into input_text
        *loaded_file = Some(file_path.to_path_buf());  // Store the loaded file path
        println!("Loaded content from {:?}", file_path);
    }
}
