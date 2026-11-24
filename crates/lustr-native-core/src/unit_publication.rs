//! Distinguish pre-publication failure from a published artifact whose final
//! directory durability could not be confirmed. Never replay an unknown effect.
use std::{fs,io,path::Path};

pub(crate) enum Publication { Durable, DurabilityUnknown(io::Error) }

pub(crate) fn publish(source:&Path,output:&Path)->io::Result<Publication>{
    publish_with_sync(source,output,|parent|parent.sync_all())
}

fn publish_with_sync(source:&Path,output:&Path,sync:impl FnOnce(&fs::File)->io::Result<()>)->io::Result<Publication>{
    let parent=output.parent().ok_or_else(||io::Error::other("object output parent required"))?;
    let directory=fs::File::open(parent)?;
    fs::File::open(source)?.sync_all()?;
    // Both names are in the caller's selected filesystem. hard_link creates
    // the complete artifact atomically and refuses an existing destination.
    fs::hard_link(source,output)?;
    Ok(match sync(&directory){Ok(())=>Publication::Durable,Err(e)=>Publication::DurabilityUnknown(e)})
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn post_publication_sync_failure_keeps_complete_object_and_reports_unknown(){
        let root=std::env::temp_dir().join(format!("lustr-publish-test-{}",std::process::id()));fs::create_dir(&root).unwrap();
        let source=root.join("source.o");let output=root.join("output.o");fs::write(&source,b"complete native object").unwrap();
        let result=publish_with_sync(&source,&output,|_|Err(io::Error::from_raw_os_error(5))).unwrap();
        assert!(matches!(result,Publication::DurabilityUnknown(_)));assert_eq!(fs::read(&output).unwrap(),b"complete native object");
        assert_eq!(publish(&source,&output).err().unwrap().kind(),io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read(&output).unwrap(),b"complete native object");fs::remove_dir_all(root).unwrap();
    }
}


/// Replacement is distinct from publish-once. All staging bytes are complete
/// and synced before the first replacement. A file set has no atomic multi-name
/// rename primitive: any partial effect or unknown durability is terminal and
/// identifies how many complete files were installed. Callers commit the link
/// archive last and report success only after the whole set is durable.
pub(crate) enum Replacement {
    Durable,
    Incomplete { replaced:usize, error:io::Error },
    DurabilityUnknown(io::Error),
}
pub(crate) fn replace_set(files:&[(&Path,&Path)])->io::Result<Replacement> {
    if files.is_empty(){return Err(io::Error::other("empty replacement set"));}
    let parent=files[0].1.parent().ok_or_else(||io::Error::other("output parent required"))?;
    let directory=fs::File::open(parent)?;
    for (source,output) in files {
        if output.parent()!=Some(parent) || source.parent().and_then(Path::parent)!=Some(parent) {
            return Err(io::Error::other("replacement staging must share the selected output filesystem"));
        }
        match fs::symlink_metadata(output) {
            Ok(meta) if !meta.file_type().is_file()=>return Err(io::Error::other("output is not a regular file")),
            Ok(_)=>{},
            Err(error) if error.kind()==io::ErrorKind::NotFound=>{},
            Err(error)=>return Err(error),
        }
        fs::File::open(source)?.sync_all()?;
    }
    for (index,(source,output)) in files.iter().enumerate() {
        if let Err(error)=fs::rename(source,output) {
            return if index==0{Err(error)}else{Ok(Replacement::Incomplete{replaced:index,error})};
        }
    }
    Ok(match directory.sync_all(){Ok(())=>Replacement::Durable,Err(error)=>Replacement::DurabilityUnknown(error)})
}
