use bevy::{prelude::*, window::WindowResolution};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use rand::Rng;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};


fn main() {
    // Create a new Bevy app and add default plugins and the EguiPlugin for UI
    App::new()

        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(628., 800.),
                title: "🗄️ Activitude file manager 🗂️".into(),
                ..default()
            }),
            ..default()
        }))
       // .add_plugins(DefaultPlugins)  // Adds default plugins (audio, window, etc.)
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
                eprintln!("Error reading directory {}", e);
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

    egui::TopBottomPanel::top("top_panel")
    .exact_height(50.0) // Set height to 50 px
    .frame(
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(153, 153, 153)) // Set background color to #999999
            .inner_margin(egui::Margin::same(10.0)),   // Add some padding
    )
    .show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space((50.0 - 20.0) / 10.0); 

            egui::Frame::none()
                .fill(egui::Color32::WHITE) 
                .rounding(egui::Rounding::same(15.0))
                .inner_margin(egui::Margin::symmetric(10.0, 5.0)) 
                .stroke(egui::Stroke::new(1.0, egui::Color32::BLACK)) 
                .show(ui, |ui| {
                    ui.allocate_ui(egui::vec2(500.0, ui.available_height()), |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut *current_dir_str)
                                .frame(false) 
                                .desired_width(500.0), 
                        );
                    });
                });
        });
    });


    // Central panel to show the main UI
    egui::CentralPanel::default()
        .frame(egui::Frame::default().inner_margin(egui::vec2(50.0, 10.0))) // Add inner margin
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| { 

                    ui.add_space(100.0);
                    // Detect right-click on the blank area of the panel
                    ui.interact(ui.max_rect(), ui.id(), egui::Sense::click()).context_menu(|ui| {
                        if ui.button("Create File").clicked() {
                            *show_file_popup = true; // Show the file creation popup
                            *input_text = String::new(); // Clear the input text
                            ui.close_menu(); // Close the context menu
                        }
                        if ui.button("Create Folder").clicked() {
                            let random_folder_name = format!("folder_{}", generate_random_number());
                            create_folder(&random_folder_name, &*current_dir_str); // Create a new folder
                            println!("Created folder: {}", random_folder_name);
                            ui.close_menu(); // Close the context menu
                        }
                    });

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
                    //####
                    // Display files and folders in the current directory
                    ui.vertical(|ui| {
                        // Use a group styled with a frame for the outlined container
                        egui::Frame::none()
                            .stroke(egui::Stroke::new(1.0, egui::Color32::BLACK)) // Black outline
                            .fill(egui::Color32::WHITE) // White background
                            .inner_margin(egui::vec2(10.0, 10.0)) // Inner padding
                            .rounding(egui::Rounding::same(10.0)) // Rounded corners
                            .show(ui, |ui| {
                                // Add the ScrollArea for the files and folders list
                                egui::ScrollArea::vertical() // Makes the container scrollable vertically
                                    .auto_shrink([false, true]) // Only shrink horizontally; keep the vertical scrolling
                                    .show(ui, |ui| {
                                        const COLUMNS: usize = 6; // Number of columns in the grid
                                        let mut current_col = 0; // Track the current column

                                        // Create a horizontal wrapped layout for the grid
                                        ui.horizontal_wrapped(|ui| {
                                            for item in files_and_folders.iter() {
                                                let item_name = item.file_name().unwrap_or_default().to_string_lossy();

                                                // Handle directory or file item
                                                ui.vertical(|ui| {
                                                    if item.is_dir() {
                                                        let logo = ui.add(
                                                            egui::ImageButton::new(
                                                                egui::Image::new(egui::include_image!("assets/folder.png"))
                                                                    .fit_to_exact_size(egui::vec2(75.0, 75.0)),
                                                            )
                                                            .frame(false),
                                                        );

                                                        if logo.clicked() {
                                                            *current_dir_str = format!("{}/{}", *current_dir_str, item_name);
                                                        }

                                                                                        // Handle right-click on the folder
                                                    if logo.secondary_clicked() {
                                                        println!("Right-clicked on folder: {}", item_name);
                                                    }

                                                    // Folder context menu with delete option
                                                    logo.context_menu(|ui| {
                                                        if ui.button("Delete Folder").clicked() {
                                                            delete_folder(item);  // Delete the folder
                                                        }
                                                    });

                                                        ui.label(item_name);
                                                    } else {
                                                        let logo = ui.add(
                                                            egui::ImageButton::new(
                                                                egui::Image::new(egui::include_image!("assets/file.png"))
                                                                    .fit_to_exact_size(egui::vec2(75.0, 75.0)),
                                                            )
                                                            .frame(false),
                                                        );

                                                        if logo.clicked() {
                                                            open_file_content(item, &mut *input_text, &mut *loaded_file);
                                                            *show_save_popup = true;
                                                        }

                                                    // File context menu with delete option
                                                    logo.context_menu(|ui| {
                                                        if ui.button("Delete File").clicked() {
                                                            delete_file(item);  // Delete the file
                                                        }
                                                    });

                                                        ui.label(item_name);
                                                    }
                                                });

                                                // Move to the next column, reset to first column if reached COLUMNS limit
                                                current_col += 1;
                                                if current_col >= COLUMNS {
                                                    current_col = 0;
                                                    ui.end_row(); // Start a new row after every 'COLUMNS' items
                                                }
                                            }
                                        });
                                    });
                            });
                    });
                   //###
                    
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
 
// Deletes a file
fn delete_file(file_path: &Path) {
    if let Err(e) = fs::remove_file(file_path) {  // Delete the file
        eprintln!("Error deleting file: {}", e);
    } else {
        println!("File deleted: {:?}", file_path);
    }
}

// Deletes a folder
fn delete_folder(folder_path: &Path) {
    if let Err(e) = fs::remove_dir_all(folder_path) {  // Delete the folder
        eprintln!("Error deleting folder: {}", e);
    } else {
        println!("Folder deleted: {:?}", folder_path);
    }
}
