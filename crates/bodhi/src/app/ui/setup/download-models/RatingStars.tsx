import { Star } from 'lucide-react';

interface RatingStarsProps {
  rating: number;
  size?: 'xs' | 'sm' | 'md';
}

export const RatingStars = ({ rating, size = 'md' }: RatingStarsProps) => {
  const sizeClass = size === 'xs' ? 'h-2.5 w-2.5' : size === 'sm' ? 'h-3 w-3' : 'h-4 w-4';
  const gapClass = size === 'xs' ? 'gap-0.5' : 'gap-1';

  return (
    <div className={`flex items-center ${gapClass}`}>
      {[1, 2, 3, 4, 5].map((star) => (
        <Star
          key={star}
          className={`${sizeClass} ${star <= rating ? 'fill-primary text-primary' : 'fill-muted text-muted-foreground'}`}
        />
      ))}
    </div>
  );
};
