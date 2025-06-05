// React equivalent for Next.js Image
import { forwardRef, ImgHTMLAttributes } from 'react';

interface ImageProps extends Omit<ImgHTMLAttributes<HTMLImageElement>, 'src'> {
  src: string;
  alt: string;
  width?: number | string;
  height?: number | string;
  priority?: boolean;
  fill?: boolean;
  sizes?: string;
  quality?: number;
  placeholder?: 'blur' | 'empty';
  blurDataURL?: string;
}

const Image = forwardRef<HTMLImageElement, ImageProps>(
  ({ src, alt, width, height, priority, fill, sizes, quality, placeholder, blurDataURL, className, style, ...props }, ref) => {
    const imgStyle: React.CSSProperties = {
      ...style,
      ...(fill && {
        position: 'absolute',
        height: '100%',
        width: '100%',
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        objectFit: 'cover',
      }),
    };

    return (
      <img
        ref={ref}
        src={src}
        alt={alt}
        width={width}
        height={height}
        className={className}
        style={imgStyle}
        loading={priority ? 'eager' : 'lazy'}
        sizes={sizes}
        {...props}
      />
    );
  }
);

Image.displayName = 'Image';

export default Image;
