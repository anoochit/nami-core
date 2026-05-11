import { useState } from 'react';

interface ToolAccordionProps {
  title: string;
  args?: any;
  result?: any;
}

export const ToolAccordion = ({ title, args, result }: ToolAccordionProps) => {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="border rounded-md overflow-hidden bg-gray-50">
      <button 
        onClick={() => setIsOpen(!isOpen)}
        className="w-full text-left p-2 font-medium flex justify-between items-center bg-gray-100 hover:bg-gray-200 transition-colors"
      >
        <span>{title}</span>
        <span className="text-sm">{isOpen ? '▲' : '▼'}</span>
      </button>
      
      {isOpen && (
        <div className="p-3 text-sm font-mono bg-white border-t overflow-x-auto">
          {args && (
            <div className="mb-2">
              <strong className="text-xs text-gray-500 uppercase">Args:</strong>
              <pre className="mt-1 p-2 bg-gray-50 rounded text-xs text-gray-500">{JSON.stringify(args, null, 2)}</pre>
            </div>
          )}
          {result && (
            <div>
              <strong className="text-xs text-gray-500 uppercase">Result:</strong>
              <pre className="mt-1 p-2 bg-gray-50 rounded text-xs text-gray-500">{JSON.stringify(result, null, 2)}</pre>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
