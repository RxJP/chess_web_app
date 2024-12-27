use std::fs;
use std::path::Path;

fn main() {
    // Define the source and destination paths
    let frontend_dir = Path::new("frontend");
    let templates_dir = Path::new("templates");
    let static_dir = Path::new("static");
    let error_dir = templates_dir.join("error");

    // Ensure the destination directories exist
    fs::create_dir_all(&error_dir).expect("Failed to create templates directory");
    fs::create_dir_all(static_dir).expect("Failed to create static directory");

    // Iterate over all files and directories in the frontend directory
    for entry in fs::read_dir(frontend_dir).expect("Failed to read frontend directory") {
        let entry = entry.expect("Failed to read entry");
        let src_path = entry.path();

        // If it's a directory, copy recursively to the static folder
        if src_path.is_dir() {
            if src_path.file_name().unwrap() == "error" {
                for entry in src_path.read_dir().unwrap() {
                    let entry = entry.unwrap();
                    fs::copy(entry.path(), error_dir.join(entry.file_name()).with_extension("html.tera")).expect("Error while copy Error templates");
                }
            }
            let dest_path = static_dir.join(src_path.file_name().unwrap());
            fs::create_dir_all(&dest_path).expect("Failed to create directory in static");
            copy_recursive(&src_path, &dest_path);
        }
        // If it's an HTML file, copy and rename to templates with a .tera extension
        else if let Some(ext) = src_path.extension() {
            if ext == "html" {
                let dest_path = templates_dir.join(src_path.file_name().unwrap()).with_extension("html.tera");
                fs::copy(&src_path, dest_path).expect("Failed to copy HTML file to templates");
            }
        }
        // Copy all other files to the static folder
        else {
            let dest_path = static_dir.join(src_path.file_name().unwrap());
            fs::copy(&src_path, dest_path).expect("Failed to copy file to static");
        }
    }
}

// Recursive function to copy directories and files
fn copy_recursive(src: &Path, dest: &Path) {
    for entry in fs::read_dir(src).expect("Failed to read directory") {
        let entry = entry.expect("Failed to read entry");
        let src_path = entry.path();
        let dest_path = dest.join(src_path.file_name().unwrap());

        if src_path.is_dir() {
            fs::create_dir_all(&dest_path).expect("Failed to create directory in static");
            copy_recursive(&src_path, &dest_path);
        } else {
            fs::copy(&src_path, &dest_path).expect("Failed to copy file to static");
        }
    }
}
