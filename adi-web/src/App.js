/*
 * Copyright 2025 Dan Mbanga
 * Licensed under the Apache License, Version 2.0
 */

import React, { useState, useCallback, useEffect } from 'react';
import { ThemeProvider } from '@mui/material/styles';
import CssBaseline from '@mui/material/CssBaseline';
import {
  Box,
  Container,
  Grid,
  Paper,
  Typography,
  Button,
  TextField,
  Tabs,
  Tab,
  LinearProgress,
  Chip,
  Alert,
  Divider,
  Card,
  CardContent,
  List,
  ListItem,
  ListItemText,
  CircularProgress,
} from '@mui/material';
import {
  CloudUpload,
  Link as LinkIcon,
  PlayArrow,
  Refresh,
  CheckCircle,
  Description,
  TableChart,
  InsertDriveFile,
  Image as ImageIcon,
  PictureAsPdf,
} from '@mui/icons-material';
import { useDropzone } from 'react-dropzone';
import { motion, AnimatePresence } from 'framer-motion';
import axios from 'axios';
import theme from './theme';
import './App.css';

const API_BASE = process.env.REACT_APP_API_BASE || 'http://localhost:8080/api/v1';

const models = [
  { value: 'read', label: 'Read', description: 'Text extraction' },
  { value: 'layout', label: 'Layout', description: 'Structure & tables' },
  { value: 'invoice', label: 'Invoice', description: 'Invoice fields' },
  { value: 'receipt', label: 'Receipt', description: 'Receipt data' },
  { value: 'id-document', label: 'ID Document', description: 'ID extraction' },
  { value: 'business-card', label: 'Business Card', description: 'Contact info' },
  { value: 'w2', label: 'W-2', description: 'Tax form' },
];

const MotionPaper = motion(Paper);
const MotionBox = motion(Box);

function App() {
  const [modelType, setModelType] = useState('read');
  const [inputMethod, setInputMethod] = useState('url');
  const [documentUrl, setDocumentUrl] = useState('');
  const [uploadedFile, setUploadedFile] = useState(null);
  const [filePreview, setFilePreview] = useState(null);
  const [loading, setLoading] = useState(false);
  const [operationId, setOperationId] = useState('');
  const [result, setResult] = useState(null);
  const [error, setError] = useState('');
  const [progress, setProgress] = useState(0);

  const onDrop = useCallback((acceptedFiles) => {
    if (acceptedFiles && acceptedFiles.length > 0) {
      const file = acceptedFiles[0];
      setUploadedFile(file);
      setError('');

      // Create preview for images and PDFs
      if (file.type.startsWith('image/')) {
        const reader = new FileReader();
        reader.onload = (e) => setFilePreview({ type: 'image', url: e.target.result });
        reader.readAsDataURL(file);
      } else if (file.type === 'application/pdf') {
        setFilePreview({ type: 'pdf', name: file.name });
      } else {
        setFilePreview({ type: 'file', name: file.name });
      }
    }
  }, []);

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    accept: {
      'application/pdf': ['.pdf'],
      'image/*': ['.png', '.jpg', '.jpeg', '.tiff', '.bmp'],
    },
    maxFiles: 1,
    maxSize: 50 * 1024 * 1024,
  });

  const analyzeDocument = async () => {
    setLoading(true);
    setError('');
    setResult(null);
    setOperationId('');
    setProgress(0);

    try {
      let response;

      if (inputMethod === 'url') {
        if (!documentUrl) {
          throw new Error('Please enter a document URL');
        }

        response = await axios.post(`${API_BASE}/analyze/${modelType}`, {
          document_url: documentUrl,
        });
      } else {
        if (!uploadedFile) {
          throw new Error('Please upload a file');
        }

        const formData = new FormData();
        formData.append('file', uploadedFile);

        response = await axios.post(`${API_BASE}/upload/${modelType}`, formData, {
          headers: {
            'Content-Type': 'multipart/form-data',
          },
          onUploadProgress: (progressEvent) => {
            const percentCompleted = Math.round(
              (progressEvent.loaded * 30) / progressEvent.total
            );
            setProgress(percentCompleted);
          },
        });
      }

      const { operation_id, status, result: immediateResult } = response.data;
      setOperationId(operation_id);
      setProgress(40);

      // Check if we got immediate results
      if (immediateResult) {
        console.log('Immediate result:', immediateResult);
        setResult(immediateResult);
        setProgress(100);
        setLoading(false);
        return;
      }

      // Start polling for results
      if (status === 'running' || status === 'notstarted') {
        pollResults(operation_id);
      } else if (status === 'succeeded') {
        // No result yet, start polling
        pollResults(operation_id);
      }
    } catch (err) {
      console.error('Analysis error:', err);
      setError(err.response?.data?.error || err.message || 'An error occurred');
      setLoading(false);
      setProgress(0);
    }
  };

  const pollResults = async (opId) => {
    const maxAttempts = 60;
    let attempts = 0;

    const poll = async () => {
      try {
        attempts++;
        console.log(`Polling attempt ${attempts} for operation:`, opId);
        
        const response = await axios.get(`${API_BASE}/results/${opId}`);
        console.log('Poll response:', response.data);
        
        const { status, result: analysisResult } = response.data;

        const progressValue = 40 + (attempts / maxAttempts) * 50;
        setProgress(Math.min(progressValue, 95));

        if (status === 'succeeded' && analysisResult) {
          console.log('Analysis succeeded with result:', analysisResult);
          setResult(analysisResult);
          setProgress(100);
          setLoading(false);
        } else if (status === 'failed') {
          setError('Analysis failed');
          setLoading(false);
          setProgress(0);
        } else if (attempts < maxAttempts) {
          setTimeout(poll, 3000); // Poll every 3 seconds
        } else {
          setError('Analysis timeout - operation may still be processing');
          setLoading(false);
          setProgress(0);
        }
      } catch (err) {
        console.error('Poll error:', err);
        if (attempts < maxAttempts) {
          setTimeout(poll, 3000);
        } else {
          setError(err.response?.data?.error || 'Error checking status');
          setLoading(false);
          setProgress(0);
        }
      }
    };

    // Start polling after a short delay
    setTimeout(poll, 2000);
  };

  const reset = () => {
    setOperationId('');
    setResult(null);
    setError('');
    setDocumentUrl('');
    setUploadedFile(null);
    setFilePreview(null);
    setLoading(false);
    setProgress(0);
  };

  // Debug: log when result changes
  useEffect(() => {
    if (result) {
      console.log('Result updated:', result);
    }
  }, [result]);

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Box
        sx={{
          minHeight: '100vh',
          background: 'linear-gradient(135deg, #f8fafc 0%, #e0e7ff 100%)',
          py: 4,
        }}
      >
        <Container maxWidth="xl">
          {/* Header */}
          <MotionBox
            initial={{ opacity: 0, y: -20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
            sx={{ mb: 4 }}
          >
            <Typography
              variant="h1"
              sx={{
                background: 'linear-gradient(135deg, #2563eb 0%, #1e40af 100%)',
                WebkitBackgroundClip: 'text',
                WebkitTextFillColor: 'transparent',
                mb: 1,
              }}
            >
              Document Intelligence
            </Typography>
            <Typography variant="body1" color="text.secondary">
              AI Document Intelligence
            </Typography>
          </MotionBox>

          <Grid container spacing={3}>
            {/* Left Column - Input & Preview */}
            <Grid item xs={12} lg={5}>
              <MotionPaper
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ duration: 0.5, delay: 0.1 }}
                elevation={3}
                sx={{ p: 3, mb: 3, bgcolor: 'background.paper' }}
              >
                {/* Model Selection */}
                <Typography variant="h6" sx={{ mb: 2, fontWeight: 600 }}>
                  1. Select Model
                </Typography>
                <Box sx={{ mb: 3, display: 'flex', flexWrap: 'wrap', gap: 1 }}>
                  {models.map((model) => (
                    <Chip
                      key={model.value}
                      label={model.label}
                      onClick={() => setModelType(model.value)}
                      color={modelType === model.value ? 'primary' : 'default'}
                      disabled={loading}
                      sx={{ 
                        cursor: 'pointer',
                        '&:hover': { transform: 'scale(1.05)' },
                        transition: 'transform 0.2s',
                      }}
                    />
                  ))}
                </Box>
                <Typography variant="caption" color="text.secondary" sx={{ display: 'block', mb: 2 }}>
                  {models.find(m => m.value === modelType)?.description || ''}
                </Typography>

                <Divider sx={{ my: 3 }} />

                {/* Input Method */}
                <Typography variant="h6" sx={{ mb: 2, fontWeight: 600 }}>
                  2. Provide Document
                </Typography>
                <Tabs
                  value={inputMethod}
                  onChange={(e, v) => setInputMethod(v)}
                  sx={{ mb: 2 }}
                >
                  <Tab label="URL" value="url" disabled={loading} />
                  <Tab label="Upload" value="upload" disabled={loading} />
                </Tabs>

                {inputMethod === 'url' ? (
                  <TextField
                    fullWidth
                    label="Document URL"
                    value={documentUrl}
                    onChange={(e) => setDocumentUrl(e.target.value)}
                    placeholder="https://example.com/document.pdf"
                    disabled={loading}
                    variant="outlined"
                    size="small"
                    InputProps={{
                      startAdornment: <LinkIcon sx={{ mr: 1, color: 'text.secondary' }} />,
                    }}
                  />
                ) : (
                  <Box>
                    <Box
                      {...getRootProps()}
                      sx={{
                        border: '2px dashed',
                        borderColor: isDragActive ? 'primary.main' : 'divider',
                        borderRadius: 2,
                        p: 3,
                        textAlign: 'center',
                        cursor: 'pointer',
                        transition: 'all 0.3s',
                        bgcolor: isDragActive ? 'rgba(37, 99, 235, 0.05)' : 'background.default',
                        '&:hover': {
                          borderColor: 'primary.main',
                          bgcolor: 'rgba(37, 99, 235, 0.08)',
                        },
                      }}
                    >
                      <input {...getInputProps()} />
                      {uploadedFile ? (
                        <Box>
                          <InsertDriveFile sx={{ fontSize: 40, color: 'primary.main', mb: 1 }} />
                          <Typography variant="body2" fontWeight={600}>{uploadedFile.name}</Typography>
                          <Typography variant="caption" color="text.secondary">
                            {(uploadedFile.size / 1024 / 1024).toFixed(2)} MB
                          </Typography>
                        </Box>
                      ) : (
                        <Box>
                          <CloudUpload sx={{ fontSize: 40, color: 'text.secondary', mb: 1 }} />
                          <Typography variant="body2" sx={{ mb: 0.5 }}>
                            Drop file or click to browse
                          </Typography>
                          <Typography variant="caption" color="text.secondary">
                            PDF, PNG, JPG, TIFF (max 50MB)
                          </Typography>
                        </Box>
                      )}
                    </Box>
                  </Box>
                )}

                <Button
                  fullWidth
                  variant="contained"
                  size="large"
                  onClick={analyzeDocument}
                  disabled={loading || (inputMethod === 'url' ? !documentUrl : !uploadedFile)}
                  startIcon={loading ? <CircularProgress size={20} color="inherit" /> : <PlayArrow />}
                  sx={{ mt: 2 }}
                >
                  {loading ? 'Analyzing...' : 'Analyze Document'}
                </Button>

                {loading && (
                  <Box sx={{ mt: 2 }}>
                    <LinearProgress
                      variant="determinate"
                      value={progress}
                      sx={{
                        height: 6,
                        borderRadius: 3,
                        bgcolor: 'rgba(37, 99, 235, 0.1)',
                        '& .MuiLinearProgress-bar': {
                          background: 'linear-gradient(90deg, #2563eb 0%, #1e40af 100%)',
                          borderRadius: 3,
                        },
                      }}
                    />
                    <Typography variant="caption" color="text.secondary" sx={{ mt: 1, display: 'block', textAlign: 'center' }}>
                      {progress < 40 ? 'Uploading...' : progress < 90 ? 'Processing...' : 'Almost done...'}
                    </Typography>
                  </Box>
                )}

                {operationId && (
                  <Alert severity="info" sx={{ mt: 2 }} icon={<CheckCircle />}>
                    <Typography variant="caption" component="div" sx={{ fontFamily: 'monospace' }}>
                      ID: {operationId}
                    </Typography>
                  </Alert>
                )}

                {error && (
                  <Alert severity="error" sx={{ mt: 2 }} onClose={() => setError('')}>
                    {error}
                  </Alert>
                )}
              </MotionPaper>

              {/* Document Preview */}
              {filePreview && (
                <MotionPaper
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  elevation={3}
                  sx={{ p: 3, bgcolor: 'background.paper' }}
                >
                  <Typography variant="h6" sx={{ mb: 2, fontWeight: 600 }}>
                    Document Preview
                  </Typography>
                  {filePreview.type === 'image' ? (
                    <Box
                      component="img"
                      src={filePreview.url}
                      alt="Document preview"
                      sx={{
                        width: '100%',
                        maxHeight: '400px',
                        objectFit: 'contain',
                        border: '1px solid',
                        borderColor: 'divider',
                        borderRadius: 2,
                        bgcolor: '#f8fafc',
                      }}
                    />
                  ) : filePreview.type === 'pdf' ? (
                    <Card variant="outlined">
                      <CardContent sx={{ textAlign: 'center', py: 4 }}>
                        <PictureAsPdf sx={{ fontSize: 60, color: 'error.main', mb: 2 }} />
                        <Typography variant="body1">{filePreview.name}</Typography>
                        <Typography variant="caption" color="text.secondary">
                          PDF document ready for analysis
                        </Typography>
                      </CardContent>
                    </Card>
                  ) : (
                    <Card variant="outlined">
                      <CardContent sx={{ textAlign: 'center', py: 4 }}>
                        <InsertDriveFile sx={{ fontSize: 60, color: 'primary.main', mb: 2 }} />
                        <Typography variant="body1">{filePreview.name}</Typography>
                      </CardContent>
                    </Card>
                  )}
                </MotionPaper>
              )}
            </Grid>

            {/* Right Column - Results */}
            <Grid item xs={12} lg={7}>
              <MotionPaper
                initial={{ opacity: 0, x: 20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ duration: 0.5, delay: 0.2 }}
                elevation={3}
                sx={{
                  p: 3,
                  minHeight: '80vh',
                  bgcolor: 'background.paper',
                }}
              >
                <Box
                  sx={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                    mb: 3,
                  }}
                >
                  <Typography variant="h6" fontWeight={600}>
                    Analysis Results
                  </Typography>
                  {result && (
                    <Button
                      variant="outlined"
                      size="small"
                      startIcon={<Refresh />}
                      onClick={reset}
                    >
                      New
                    </Button>
                  )}
                </Box>

                <AnimatePresence mode="wait">
                  {!result && !loading && (
                    <MotionBox
                      key="empty"
                      initial={{ opacity: 0 }}
                      animate={{ opacity: 1 }}
                      exit={{ opacity: 0 }}
                      sx={{
                        display: 'flex',
                        flexDirection: 'column',
                        alignItems: 'center',
                        justifyContent: 'center',
                        height: '60vh',
                        color: 'text.secondary',
                      }}
                    >
                      <Description sx={{ fontSize: 80, mb: 2, opacity: 0.2 }} />
                      <Typography variant="h6" gutterBottom color="text.secondary">
                        No results yet
                      </Typography>
                      <Typography variant="body2" color="text.secondary">
                        Analyze a document to see extracted data
                      </Typography>
                    </MotionBox>
                  )}

                  {loading && (
                    <MotionBox
                      key="loading"
                      initial={{ opacity: 0 }}
                      animate={{ opacity: 1 }}
                      exit={{ opacity: 0 }}
                      sx={{
                        display: 'flex',
                        flexDirection: 'column',
                        alignItems: 'center',
                        justifyContent: 'center',
                        height: '60vh',
                      }}
                    >
                      <CircularProgress size={80} thickness={3} sx={{ mb: 3 }} />
                      <Typography variant="h6" gutterBottom>
                        Processing Document
                      </Typography>
                      <Typography variant="body2" color="text.secondary">
                        Please wait while we analyze your document...
                      </Typography>
                      <Typography variant="caption" color="text.secondary" sx={{ mt: 1 }}>
                        This may take 10-30 seconds
                      </Typography>
                    </MotionBox>
                  )}

                  {result && (
                    <MotionBox
                      key="results"
                      initial={{ opacity: 0, y: 20 }}
                      animate={{ opacity: 1, y: 0 }}
                      exit={{ opacity: 0 }}
                      transition={{ duration: 0.5 }}
                    >
                      {/* Summary Stats */}
                      <Grid container spacing={2} sx={{ mb: 3 }}>
                        <Grid item xs={4}>
                          <Card sx={{ bgcolor: 'rgba(37, 99, 235, 0.08)', border: '2px solid #2563eb30' }}>
                            <CardContent sx={{ textAlign: 'center', py: 2 }}>
                              <Typography variant="h3" color="primary.main" fontWeight="bold">
                                {result.pages?.length || 0}
                              </Typography>
                              <Typography variant="body2" color="text.secondary">
                                Pages
                              </Typography>
                            </CardContent>
                          </Card>
                        </Grid>
                        <Grid item xs={4}>
                          <Card sx={{ bgcolor: 'rgba(100, 116, 139, 0.08)', border: '2px solid #64748b30' }}>
                            <CardContent sx={{ textAlign: 'center', py: 2 }}>
                              <Typography variant="h3" color="secondary.main" fontWeight="bold">
                                {result.tables?.length || 0}
                              </Typography>
                              <Typography variant="body2" color="text.secondary">
                                Tables
                              </Typography>
                            </CardContent>
                          </Card>
                        </Grid>
                        <Grid item xs={4}>
                          <Card sx={{ bgcolor: 'rgba(16, 185, 129, 0.08)', border: '2px solid #10b98130' }}>
                            <CardContent sx={{ textAlign: 'center', py: 2 }}>
                              <Typography variant="h3" sx={{ color: '#10b981' }} fontWeight="bold">
                                {result.content?.length || 0}
                              </Typography>
                              <Typography variant="body2" color="text.secondary">
                                Chars
                              </Typography>
                            </CardContent>
                          </Card>
                        </Grid>
                      </Grid>

                      {/* Model Info */}
                      <Box sx={{ mb: 3, display: 'flex', gap: 1, flexWrap: 'wrap', alignItems: 'center' }}>
                        <Chip
                          label={result.model_id || 'Unknown Model'}
                          size="small"
                          color="primary"
                          variant="outlined"
                        />
                        <Chip
                          label="Completed"
                          size="small"
                          icon={<CheckCircle />}
                          color="success"
                        />
                      </Box>

                      <Divider sx={{ my: 3 }} />

                      {/* Extracted Text */}
                      {result.content && result.content.trim().length > 0 ? (
                        <Box sx={{ mb: 3 }}>
                          <Typography variant="h6" gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1, fontWeight: 600 }}>
                            <Description color="primary" />
                            Extracted Text
                          </Typography>
                          <Paper
                            elevation={0}
                            sx={{
                              p: 3,
                              maxHeight: '400px',
                              overflow: 'auto',
                              bgcolor: '#f8fafc',
                              border: '2px solid #e2e8f0',
                              borderRadius: 2,
                              fontFamily: '"Monaco", "Menlo", "Courier New", monospace',
                              fontSize: '0.85rem',
                              lineHeight: 1.8,
                              whiteSpace: 'pre-wrap',
                              wordBreak: 'break-word',
                              color: '#0f172a',
                            }}
                          >
                            {result.content}
                          </Paper>
                          <Box sx={{ mt: 1, display: 'flex', gap: 2 }}>
                            <Chip 
                              label={`${result.content.split('\n').filter(l => l.trim()).length} lines`} 
                              size="small" 
                              variant="outlined"
                            />
                            <Chip 
                              label={`${result.content.split(/\s+/).filter(w => w).length} words`} 
                              size="small" 
                              variant="outlined"
                            />
                          </Box>
                        </Box>
                      ) : (
                        <Alert severity="info" sx={{ mb: 3 }}>
                          <Typography variant="body2">
                            No text content extracted. The document may be empty or contain only images/tables.
                          </Typography>
                        </Alert>
                      )}

                      {/* Pages Info */}
                      {result.pages && result.pages.length > 0 && (
                        <Box sx={{ mb: 3 }}>
                          <Typography variant="h6" gutterBottom sx={{ fontWeight: 600 }}>
                            Pages ({result.pages.length})
                          </Typography>
                          <Grid container spacing={1}>
                            {result.pages.map((page, idx) => (
                              <Grid item xs={6} sm={4} md={3} key={idx}>
                                <Card
                                  variant="outlined"
                                  sx={{
                                    transition: 'all 0.2s',
                                    '&:hover': {
                                      borderColor: 'primary.main',
                                      transform: 'translateY(-2px)',
                                      boxShadow: 2,
                                    },
                                  }}
                                >
                                  <CardContent sx={{ p: 2 }}>
                                    <Typography variant="subtitle2" fontWeight={600} color="primary.main">
                                      Page {page.page_number}
                                    </Typography>
                                    <Typography variant="caption" display="block" color="text.secondary">
                                      {page.width?.toFixed(0)} × {page.height?.toFixed(0)}px
                                    </Typography>
                                    <Typography variant="caption" display="block" color="text.secondary">
                                      {page.word_count} words, {page.line_count} lines
                                    </Typography>
                                  </CardContent>
                                </Card>
                              </Grid>
                            ))}
                          </Grid>
                        </Box>
                      )}

                      {/* Tables Info */}
                      {result.tables && result.tables.length > 0 && (
                        <Box>
                          <Typography variant="h6" gutterBottom sx={{ fontWeight: 600 }}>
                            Tables ({result.tables.length})
                          </Typography>
                          <List>
                            {result.tables.map((table, idx) => (
                              <ListItem
                                key={idx}
                                sx={{
                                  border: '1px solid',
                                  borderColor: 'divider',
                                  borderRadius: 2,
                                  mb: 1,
                                  bgcolor: 'background.default',
                                }}
                              >
                                <TableChart color="secondary" sx={{ mr: 2 }} />
                                <ListItemText
                                  primary={`Table ${idx + 1}`}
                                  secondary={`${table.row_count} rows × ${table.column_count} columns (${table.cell_count} cells)`}
                                  primaryTypographyProps={{ fontWeight: 600 }}
                                />
                              </ListItem>
                            ))}
                          </List>
                        </Box>
                      )}

                      {/* Debug Info (can be removed in production) */}
                      {result && (
                        <Box sx={{ mt: 3, pt: 3, borderTop: '1px solid', borderColor: 'divider' }}>
                          <Typography variant="caption" color="text.secondary" sx={{ display: 'block' }}>
                            Debug Info: {JSON.stringify(result).length} bytes received
                          </Typography>
                          <Typography variant="caption" color="text.secondary" sx={{ display: 'block' }}>
                            Has content: {result.content ? 'Yes' : 'No'}
                          </Typography>
                          <Typography variant="caption" color="text.secondary" sx={{ display: 'block' }}>
                            Content length: {result.content?.length || 0}
                          </Typography>
                        </Box>
                      )}
                    </MotionBox>
                  )}
                </AnimatePresence>
              </MotionPaper>
            </Grid>
          </Grid>

          {/* Footer */}
          <Box sx={{ mt: 4, textAlign: 'center' }}>
            <Typography variant="caption" color="text.secondary">
              © 2025 Dan Mbanga • Licensed under Apache 2.0
              <br />
              Built with Rust, gRPC, PostgreSQL, React, and Material-UI
            </Typography>
          </Box>
        </Container>
      </Box>
    </ThemeProvider>
  );
}

export default App;
