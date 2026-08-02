import React, { useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/tauri";
import { ImageIcon, VideoIcon, MusicIcon, FileTextIcon, ArchiveIcon, FileIcon } from "../icons";

interface FileThumbnailProps {
  file: {
    id: string;
    original_name: string;
    file_type: string;
    category: string;
    path: string;
    recovered_path: string;
    status: string;
  };
}

const FileThumbnail: React.FC<FileThumbnailProps> = ({ file }) => {
  const [imgError, setImgError] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  // Check if this is an image file
  const imageExtensions = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff", "heic"];
  const isImage = imageExtensions.includes(file.file_type.toLowerCase());

  // Determine which path to use for the thumbnail
  const thumbnailPath = file.recovered_path || file.path;
  const isRecovered = file.status.includes("recovered") || file.status.includes("repaired");

  // Only show real image if it's recovered (has a valid path)
  const shouldShowImage = isImage && isRecovered && thumbnailPath && !imgError;

  const getImageSrc = () => {
    try {
      return convertFileSrc(thumbnailPath);
    } catch (e) {
      console.error("Failed to convert file path:", e);
      return "";
    }
  };

  if (shouldShowImage) {
    return (
      <div className="file-thumbnail image-thumbnail">
        {isLoading && (
          <div className="thumbnail-loader">
            <div className="spinner" />
          </div>
        )}
        <img
          src={getImageSrc()}
          alt={file.original_name}
          className="thumbnail-image"
          style={{ display: isLoading ? "none" : "block" }}
          onLoad={() => setIsLoading(false)}
          onError={() => {
            setImgError(true);
            setIsLoading(false);
          }}
        />
        {imgError && (
          <div className="thumbnail-fallback">
            <ImageIcon size={32} color="var(--text-muted)" />
          </div>
        )}
      </div>
    );
  }

  // For non-image files or failed images, show appropriate icon
  const getCategoryIcon = () => {
    switch (file.category) {
      case "images": return <ImageIcon size={32} color="var(--accent)" />;
      case "videos": return <VideoIcon size={32} color="var(--info)" />;
      case "audio": return <MusicIcon size={32} color="var(--warning)" />;
      case "documents": return <FileTextIcon size={32} color="var(--success)" />;
      case "archives": return <ArchiveIcon size={32} color="var(--text-muted)" />;
      default: return <FileIcon size={32} color="var(--text-muted)" />;
    }
  };

  return (
    <div className="file-thumbnail">
      {getCategoryIcon()}
    </div>
  );
};

export default FileThumbnail;
