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
