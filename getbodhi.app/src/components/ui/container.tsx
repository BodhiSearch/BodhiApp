interface ContainerProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode;
}

export function Container({ children, className, ...props }: ContainerProps) {
  return (
    <div 
      className={`max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 ${className || ''}`}
      {...props}
    >
      {children}
    </div>
  );
} 