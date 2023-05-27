import '@/styles/globals.css'

export const metadata = {
  title: 'nextjs streaming demo using suspense fallback'
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en" className="[color-scheme:dark]">
      <head>
        <link rel="icon" href="data:;base64,iVBORw0KGgo=" />
      </head>
      <body className="overflow-y-scroll bg-gray-1100">

        <div className="lg:pl-0">
          <div className="mx-auto max-w-4xl space-y-8 px-2 pt-20 lg:py-8 lg:px-8">


            <div className="rounded-lg bg-vc-border-gradient p-px shadow-lg shadow-black/20">
              <div className="rounded-lg bg-black p-3.5 lg:p-6">

                <div className='space-y-9'>
                  <div>
                    <div className="prose prose-sm prose-invert mb-8 max-w-none">
                      <ul>
                        <li>
                          Primary product information is loaded first as part of the initial
                          response.
                        </li>
                        <li>
                          Secondary, more personalized details (that might be slower) like
                          ship date, other recommended products, and customer reviews are
                          progressively streamed in.
                        </li>
                      </ul>
                    </div>
                    <div className='relative rounded-lg border border-dashed p-3 lg:p-5 border-gray-700'>
                      <div className="absolute -top-2.5 flex gap-x-1 text-[9px] uppercase leading-4 tracking-widest left-3 lg:left-5">
                        <div className="rounded-full px-1.5 shadow-[0_0_1px_3px_black] bg-gray-800 text-gray-300">Demo</div>
                      </div>
                      <div className='space-y-10'>
                        {children}
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </body>
    </html >
  )
}
