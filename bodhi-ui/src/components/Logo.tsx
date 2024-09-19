import Image from 'next/image';

export default function Logo() {
  return (
    <div className="flex items-center space-x-4 justify-center">
      <Image src="/bodhi-logo.png" alt="Bodhi Logo" width={50} height={50} />
      <h1 className="text-3xl sm:text-4xl font-bold text-primary">Bodhi</h1>
    </div>
  );
}
