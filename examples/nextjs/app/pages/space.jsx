import Image from "next/image";

import spacePic from "../public/space.jpg";

export default function LocalImage() {
  return (
    <>
      <Image src={spacePic} alt="space" fill placeholder="blur" 
            width={2180} height={697} />
    </>
  );
}
