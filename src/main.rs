use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

fn main() {
    App::new()
        // Add default plugins for Bevy (e.g., window, asset loading, etc.)
        .add_plugins(DefaultPlugins)
        // Add Egui plugin for GUI
        .add_plugins(EguiPlugin)
        // Set background color for the application window
        .insert_resource(ClearColor(Color::rgb(214.0 / 255.0, 204.0 / 255.0, 185.0 / 255.0)))
        // Add system that will handle UI rendering and interactions
        .add_systems(Update, ui_system)
        .run();
}

fn ui_system(
    mut contexts: EguiContexts,
    mut input_text: Local<String>,
    mut file_name: Local<String>,
    mut folder_name: Local<String>,
    mut show_file_popup: Local<bool>,
    mut show_folder_popup: Local<bool>,
    mut files_and_folders: Local<Vec<PathBuf>>,
    mut loaded_file: Local<Option<PathBuf>>,
    mut current_dir_str: Local<String>,
) {
    // Set initial directory path to "./root" if it's empty
    if current_dir_str.is_empty() {
        *current_dir_str = String::from("./root");
    }

    let dir_path = Path::new(current_dir_str.as_str());

    // Read contents of the current directory if it exists
    if dir_path.exists() {
        let entries = match fs::read_dir(dir_path) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("Error reading directory: {}", e);
                return;
            }
        };

        let mut file_paths = Vec::new();
        let mut folder_paths = Vec::new();

        // Separate files and folders into different vectors
        for entry in entries.filter_map(|entry| entry.ok()) {
            let path = entry.path();
            if path.is_file() {
                file_paths.push(path);
            } else if path.is_dir() {
                folder_paths.push(path);
            }
        }

        // Combine both folders and files into a single list to display
        *files_and_folders = [folder_paths, file_paths].concat();
    }

    // Define the UI layout using Egui
    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(contexts.ctx_mut(), |ui| {
            ui.vertical(|ui| {
                // Display and edit the current directory
                ui.horizontal(|ui| {
                    ui.label("Current Directory: ");
                    ui.text_edit_singleline(&mut *current_dir_str); 
                });

                // Button to open the file creation popup
                if ui.button("Create File").clicked() {
                    *show_file_popup = true;
                }

                // Button to open the folder creation popup
                if ui.button("Create Folder").clicked() {
                    *show_folder_popup = true;
                }

                // Show file creation popup if triggered
                if *show_file_popup {
                    show_file_name_popup(ui, &mut *file_name, &mut *show_file_popup, &*current_dir_str);
                }

                // Show folder creation popup if triggered
                if *show_folder_popup {
                    show_folder_name_popup(ui, &mut *folder_name, &mut *show_folder_popup, &*current_dir_str);
                }

                // Display files and folders in the current directory
                ui.horizontal(|ui| {
                    ui.label("Files and Folders in current directory: ");
                    for item in files_and_folders.iter() {
                        let item_name = item.file_name().unwrap_or_default().to_string_lossy();
                        if item.is_dir() {
                            // Click on a folder to navigate into it
                            if ui.button(format!("Folder: {}", item_name)).clicked() {
                                *current_dir_str = format!("{}/{}", *current_dir_str, item_name);
                                println!("Updated current directory to: {}", *current_dir_str);
                            }
                        } else {
                            // Click on a file to open and display its content
                            if ui.button(format!("File: {}", item_name)).clicked() {
                                open_file_content(item, &mut *input_text, &mut *loaded_file);
                            }
                        }
                    }
                });

                // Display the loaded file name if any file is loaded
                if let Some(ref path) = *loaded_file {
                    ui.label(format!("Loaded File: {}", path.file_name().unwrap_or_default().to_string_lossy()));
                } else {
                    ui.label("No file loaded.");
                }

                // Add space for better layout
                let top_half_height = ui.available_height() / 2.0;
                ui.add_space(top_half_height);

                // Buttons for saving content or cancelling editing
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        save_content(&mut *input_text, &*loaded_file);
                    }
                    if ui.button("Cancel").clicked() {
                        *input_text = String::new();
                        *loaded_file = None;
                    }
                });

                // Input field for editing content in the loaded file
                let input_size = egui::vec2(ui.available_width(), ui.available_height());
                ui.add_sized(input_size, egui::TextEdit::multiline(&mut *input_text).desired_rows(10));
            });
        });
}

// Popup to enter a file name for creation
fn show_file_name_popup(ui: &mut egui::Ui, file_name: &mut String, show_popup: &mut bool, current_dir_str: &str) {
    ui.horizontal(|ui| {
        ui.label("Enter file name: ");
        ui.text_edit_singleline(file_name);
    });

    // Create the file if the "Create File" button is clicked
    if ui.button("Create File").clicked() {
        if !file_name.is_empty() {
            create_file(file_name, current_dir_str);
            file_name.clear();
            *show_popup = false;
        }
    }

    // Close the popup if the "Cancel" button is clicked
    if ui.button("Cancel").clicked() {
        *show_popup = false;
    }
}

// Popup to enter a folder name for creation
fn show_folder_name_popup(ui: &mut egui::Ui, folder_name: &mut String, show_popup: &mut bool, current_dir_str: &str) {
    ui.horizontal(|ui| {
        ui.label("Enter folder name: ");
        ui.text_edit_singleline(folder_name);
    });

    // Create the folder if the "Create Folder" button is clicked
    if ui.button("Create Folder").clicked() {
        if !folder_name.is_empty() {
            create_folder(folder_name, current_dir_str);
            folder_name.clear();
            *show_popup = false;
        }
    }

    // Close the popup if the "Cancel" button is clicked
    if ui.button("Cancel").clicked() {
        *show_popup = false;
    }
}

// Function to create a file in the specified directory
fn create_file(file_name: &str, current_dir_str: &str) {
    let full_path = Path::new(current_dir_str).join(file_name); 
    if let Err(e) = File::create(&full_path) {
        eprintln!("Error creating file: {}", e);
    } else {
        println!("File created: {:?}", full_path);
    }
}

// Function to create a folder in the specified directory
fn create_folder(folder_name: &str, current_dir_str: &str) {
    let folder_path = Path::new(current_dir_str).join(folder_name); 
    if let Err(e) = fs::create_dir(&folder_path) {
        eprintln!("Error creating folder: {}", e);
    } else {
        println!("Folder created: {:?}", folder_path);
    }
}

// Function to save the content of the input field to the loaded file
fn save_content(input_text: &mut String, loaded_file: &Option<PathBuf>) {
    if let Some(ref file_path) = *loaded_file {
        let mut file = match File::create(file_path) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Error opening file for writing: {}", e);
                return;
            }
        };

        // Write the content to the file
        if let Err(e) = file.write_all(input_text.as_bytes()) {
            eprintln!("Error writing to file: {}", e);
        } else {
            println!("Content saved to {:?}", file_path);
        }
    } else {
        eprintln!("No file loaded. Cannot save content.");
    }
}

// Function to load the content of a file into the input field
fn open_file_content(file_path: &Path, input_text: &mut String, loaded_file: &mut Option<PathBuf>) {
    let mut file = File::open(file_path).unwrap_or_else(|_| {
        eprintln!("Error opening file: {:?}", file_path);
        std::process::exit(1);
    });

    let mut content = String::new();
    if let Err(e) = file.read_to_string(&mut content) {
        eprintln!("Error reading file: {:?}", e);
    } else {
        *input_text = content;
        *loaded_file = Some(file_path.to_path_buf());
        println!("Loaded content from {:?}", file_path);
    }
}
