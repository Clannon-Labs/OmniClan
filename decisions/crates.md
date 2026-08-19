## `uuid = { version = "1.24.1", features = ["v7"] }`
> With v7, we can order the filename by time of creation
> and id will stay unique too, i didn't prefer v4 here because
> we might later need to retrieve the file by the creation date/time
> i didn't prefer v8 (which would have enabled me to embed userid into
> filename which would have been good if we needed to store all videos
> into a single row and needed to retrieve video by userid)
> but videos of each user are stored separately, and db will do the filtering
> so we dont need v8 here.
