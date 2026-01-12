#!/usr/bin/env python3
"""Generate CEO Executive Summary PDF for Roea AI"""

from reportlab.lib import colors
from reportlab.lib.pagesizes import letter
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.units import inch
from reportlab.platypus import (
    SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle,
    PageBreak, ListFlowable, ListItem, HRFlowable
)
from reportlab.lib.enums import TA_CENTER, TA_JUSTIFY, TA_LEFT
from datetime import datetime

def create_ceo_document():
    doc = SimpleDocTemplate(
        "/home/user/roea-ai/docs/CEO_Executive_Summary_Roea_AI.pdf",
        pagesize=letter,
        rightMargin=0.75*inch,
        leftMargin=0.75*inch,
        topMargin=0.75*inch,
        bottomMargin=0.75*inch
    )

    styles = getSampleStyleSheet()

    # Custom styles
    title_style = ParagraphStyle(
        'CustomTitle',
        parent=styles['Heading1'],
        fontSize=28,
        spaceAfter=6,
        alignment=TA_CENTER,
        textColor=colors.HexColor('#1a365d'),
        fontName='Helvetica-Bold'
    )

    subtitle_style = ParagraphStyle(
        'Subtitle',
        parent=styles['Normal'],
        fontSize=14,
        alignment=TA_CENTER,
        textColor=colors.HexColor('#4a5568'),
        spaceAfter=30
    )

    section_style = ParagraphStyle(
        'SectionHeader',
        parent=styles['Heading2'],
        fontSize=16,
        spaceBefore=20,
        spaceAfter=12,
        textColor=colors.HexColor('#2c5282'),
        fontName='Helvetica-Bold'
    )

    subsection_style = ParagraphStyle(
        'SubsectionHeader',
        parent=styles['Heading3'],
        fontSize=13,
        spaceBefore=15,
        spaceAfter=8,
        textColor=colors.HexColor('#2d3748'),
        fontName='Helvetica-Bold'
    )

    body_style = ParagraphStyle(
        'CustomBody',
        parent=styles['Normal'],
        fontSize=11,
        alignment=TA_JUSTIFY,
        spaceAfter=10,
        leading=14
    )

    highlight_style = ParagraphStyle(
        'Highlight',
        parent=styles['Normal'],
        fontSize=11,
        alignment=TA_CENTER,
        textColor=colors.HexColor('#2c5282'),
        backColor=colors.HexColor('#ebf8ff'),
        spaceBefore=15,
        spaceAfter=15,
        borderPadding=10
    )

    story = []

    # Title Page
    story.append(Spacer(1, 1.5*inch))
    story.append(Paragraph("ROEA AI", title_style))
    story.append(Paragraph("רועה AI", ParagraphStyle('Hebrew', parent=subtitle_style, fontSize=18)))
    story.append(Spacer(1, 0.3*inch))
    story.append(HRFlowable(width="40%", thickness=2, color=colors.HexColor('#2c5282')))
    story.append(Spacer(1, 0.3*inch))
    story.append(Paragraph("Executive Summary", subtitle_style))
    story.append(Paragraph("AI Agent Orchestration Platform", subtitle_style))
    story.append(Spacer(1, 1*inch))
    story.append(Paragraph(f"Confidential | {datetime.now().strftime('%B %Y')}",
                          ParagraphStyle('Date', parent=subtitle_style, fontSize=10)))
    story.append(PageBreak())

    # Executive Overview
    story.append(Paragraph("Executive Overview", section_style))
    story.append(Paragraph(
        "<b>Roea AI</b> (Hebrew: רועה - 'The Herder') is an enterprise-grade AI agent orchestration platform "
        "that manages, coordinates, and executes multiple specialized AI coding agents. The platform enables "
        "organizations to leverage AI for software development at scale while maintaining security, "
        "auditability, and operational control.",
        body_style
    ))

    story.append(Paragraph(
        "The system addresses a critical gap in the market: while AI coding assistants like Claude and GPT "
        "are powerful individually, enterprises need infrastructure to deploy and manage multiple agents "
        "working simultaneously on complex codebases with proper governance.",
        body_style
    ))

    # Value Proposition Box
    value_data = [
        ['Core Value Proposition'],
        ['"Orchestrate AI coding agents at enterprise scale with security, visibility, and control"']
    ]
    value_table = Table(value_data, colWidths=[6*inch])
    value_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), colors.HexColor('#2c5282')),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('ALIGN', (0, 0), (-1, -1), 'CENTER'),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, 0), 12),
        ('FONTSIZE', (0, 1), (-1, 1), 11),
        ('BACKGROUND', (0, 1), (-1, 1), colors.HexColor('#ebf8ff')),
        ('FONTNAME', (0, 1), (-1, 1), 'Helvetica-Oblique'),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 12),
        ('TOPPADDING', (0, 0), (-1, -1), 12),
        ('BOX', (0, 0), (-1, -1), 1, colors.HexColor('#2c5282')),
    ]))
    story.append(Spacer(1, 0.2*inch))
    story.append(value_table)
    story.append(Spacer(1, 0.2*inch))

    # Market Opportunity
    story.append(Paragraph("Market Opportunity", section_style))
    story.append(Paragraph(
        "The AI-assisted software development market is experiencing explosive growth. Key market drivers include:",
        body_style
    ))

    market_points = [
        "<b>$150B+ TAM</b> - Enterprise software development spending that can be augmented by AI",
        "<b>10x productivity gains</b> reported by early AI coding assistant adopters",
        "<b>Developer shortage</b> - 4M+ unfilled developer positions globally",
        "<b>Enterprise adoption barriers</b> - Security, compliance, and governance concerns limiting AI adoption"
    ]
    for point in market_points:
        story.append(Paragraph("• " + point, body_style))

    # Problem Statement
    story.append(Paragraph("The Problem We Solve", section_style))

    problems = [
        ["Challenge", "Current State", "Roea Solution"],
        ["Multi-Agent Coordination", "Manual handoffs, conflicts, duplicate work", "Automated orchestration with task queuing"],
        ["Security & Secrets", "API keys exposed in prompts", "Age encryption for all credentials"],
        ["Auditability", "No history of AI actions", "Complete audit trail via Fossil SCM"],
        ["Scalability", "Single-user, single-task", "Kubernetes-native, multi-agent parallel execution"],
        ["Cost Control", "Unbounded API spending", "Per-task budgets and model routing"]
    ]

    prob_table = Table(problems, colWidths=[1.5*inch, 2.25*inch, 2.25*inch])
    prob_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), colors.HexColor('#2c5282')),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, -1), 9),
        ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
        ('VALIGN', (0, 0), (-1, -1), 'MIDDLE'),
        ('GRID', (0, 0), (-1, -1), 0.5, colors.HexColor('#cbd5e0')),
        ('BACKGROUND', (0, 1), (-1, -1), colors.HexColor('#f7fafc')),
        ('ROWBACKGROUNDS', (0, 1), (-1, -1), [colors.HexColor('#f7fafc'), colors.white]),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 8),
        ('TOPPADDING', (0, 0), (-1, -1), 8),
        ('LEFTPADDING', (0, 0), (-1, -1), 6),
    ]))
    story.append(Spacer(1, 0.15*inch))
    story.append(prob_table)

    # Product Overview
    story.append(PageBreak())
    story.append(Paragraph("Product Overview", section_style))

    story.append(Paragraph("Platform Architecture", subsection_style))
    story.append(Paragraph(
        "Roea AI is built on a modern, cloud-native architecture designed for enterprise deployment:",
        body_style
    ))

    arch_components = [
        "<b>Web Dashboard</b> - React-based kanban interface for task management and monitoring",
        "<b>Orchestration Engine</b> - Go-based backend managing task lifecycle and agent coordination",
        "<b>Execution Layer</b> - Pluggable backends supporting local, Kubernetes, and VM execution",
        "<b>Storage Layer</b> - Fossil SCM providing versioned, auditable data storage in a single file",
        "<b>Security Layer</b> - Age encryption for secrets with X25519 key management"
    ]
    for comp in arch_components:
        story.append(Paragraph("• " + comp, body_style))

    story.append(Paragraph("Specialized AI Agents", subsection_style))
    story.append(Paragraph(
        "The platform includes five purpose-built agents, each optimized for specific development tasks:",
        body_style
    ))

    agents_data = [
        ["Agent", "Purpose", "Use Cases"],
        ["General Coder", "Full-stack development", "New features, integrations, refactoring"],
        ["Bug Fixer", "Debugging & resolution", "Issue triage, root cause analysis, patches"],
        ["Code Reviewer", "Quality assurance", "PR reviews, security audits, best practices"],
        ["Docs Writer", "Documentation", "API docs, READMEs, architecture guides"],
        ["Test Writer", "Test automation", "Unit tests, integration tests, E2E coverage"]
    ]

    agents_table = Table(agents_data, colWidths=[1.3*inch, 1.7*inch, 3*inch])
    agents_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), colors.HexColor('#38a169')),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, -1), 9),
        ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
        ('VALIGN', (0, 0), (-1, -1), 'MIDDLE'),
        ('GRID', (0, 0), (-1, -1), 0.5, colors.HexColor('#cbd5e0')),
        ('ROWBACKGROUNDS', (0, 1), (-1, -1), [colors.HexColor('#f0fff4'), colors.white]),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 8),
        ('TOPPADDING', (0, 0), (-1, -1), 8),
        ('LEFTPADDING', (0, 0), (-1, -1), 6),
    ]))
    story.append(agents_table)

    # Key Differentiators
    story.append(Paragraph("Key Differentiators", section_style))

    diff_data = [
        ["Feature", "Roea AI", "Traditional Tools"],
        ["Deployment", "Single binary, zero dependencies", "Complex multi-service setup"],
        ["Data Storage", "Single Fossil file with versioning", "PostgreSQL + Redis + object storage"],
        ["Secret Management", "Built-in age encryption", "External vault integration required"],
        ["Execution Backends", "Local + K8s + VM native", "Usually cloud-only"],
        ["Agent Communication", "MCP protocol standard", "Proprietary APIs"],
        ["Audit Trail", "Complete via Fossil SCM", "Manual logging setup"]
    ]

    diff_table = Table(diff_data, colWidths=[1.5*inch, 2.25*inch, 2.25*inch])
    diff_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), colors.HexColor('#805ad5')),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, -1), 9),
        ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
        ('VALIGN', (0, 0), (-1, -1), 'MIDDLE'),
        ('GRID', (0, 0), (-1, -1), 0.5, colors.HexColor('#cbd5e0')),
        ('ROWBACKGROUNDS', (0, 1), (-1, -1), [colors.HexColor('#faf5ff'), colors.white]),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 8),
        ('TOPPADDING', (0, 0), (-1, -1), 8),
        ('LEFTPADDING', (0, 0), (-1, -1), 6),
    ]))
    story.append(diff_table)

    # Technology Stack
    story.append(PageBreak())
    story.append(Paragraph("Technology Foundation", section_style))

    tech_data = [
        ["Layer", "Technology", "Rationale"],
        ["Backend", "Go 1.22, Gin Framework", "Performance, concurrency, single binary deployment"],
        ["Frontend", "Next.js 14, React 18, TypeScript", "Modern DX, type safety, server components"],
        ["Database", "Fossil SCM (SQLite-backed)", "Zero dependencies, versioning, single-file backup"],
        ["Encryption", "Age (X25519)", "Modern, audited, simple key management"],
        ["Protocol", "MCP (Model Context Protocol)", "Standard agent communication interface"],
        ["Container", "Docker, Kubernetes", "Cloud-native, horizontal scaling"]
    ]

    tech_table = Table(tech_data, colWidths=[1.2*inch, 2.3*inch, 2.5*inch])
    tech_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), colors.HexColor('#dd6b20')),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, -1), 9),
        ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
        ('VALIGN', (0, 0), (-1, -1), 'MIDDLE'),
        ('GRID', (0, 0), (-1, -1), 0.5, colors.HexColor('#cbd5e0')),
        ('ROWBACKGROUNDS', (0, 1), (-1, -1), [colors.HexColor('#fffaf0'), colors.white]),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 8),
        ('TOPPADDING', (0, 0), (-1, -1), 8),
        ('LEFTPADDING', (0, 0), (-1, -1), 6),
    ]))
    story.append(tech_table)

    # Development Status
    story.append(Paragraph("Development Status", section_style))

    status_data = [
        ["Phase", "Description", "Status"],
        ["Phase 1", "Core infrastructure (Fossil, encryption, types)", "Complete"],
        ["Phase 2", "API & local execution", "Complete"],
        ["Phase 3", "MCP server & agent integration", "Complete"],
        ["Phase 4", "Web dashboard & monitoring", "In Progress"],
        ["Phase 5", "Kubernetes & VM executors", "Planned"],
        ["Phase 6", "Multi-model routing & cost tracking", "Planned"]
    ]

    status_table = Table(status_data, colWidths=[1.2*inch, 3.3*inch, 1.5*inch])
    status_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), colors.HexColor('#2c5282')),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, -1), 9),
        ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
        ('ALIGN', (2, 0), (2, -1), 'CENTER'),
        ('VALIGN', (0, 0), (-1, -1), 'MIDDLE'),
        ('GRID', (0, 0), (-1, -1), 0.5, colors.HexColor('#cbd5e0')),
        ('BACKGROUND', (2, 1), (2, 3), colors.HexColor('#c6f6d5')),
        ('BACKGROUND', (2, 4), (2, 4), colors.HexColor('#fefcbf')),
        ('BACKGROUND', (2, 5), (2, 6), colors.HexColor('#e2e8f0')),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 8),
        ('TOPPADDING', (0, 0), (-1, -1), 8),
        ('LEFTPADDING', (0, 0), (-1, -1), 6),
    ]))
    story.append(status_table)

    # Code Metrics
    story.append(Paragraph("Code Metrics", subsection_style))

    metrics = [
        "<b>Backend:</b> ~2,600 lines of Go across 9 packages",
        "<b>Frontend:</b> 7 React/TypeScript components",
        "<b>Built-in Agents:</b> 5 specialized agents with documentation",
        "<b>Test Coverage:</b> Core business logic covered",
        "<b>Build Targets:</b> 20+ Makefile targets for CI/CD"
    ]
    for m in metrics:
        story.append(Paragraph("• " + m, body_style))

    # Business Model
    story.append(PageBreak())
    story.append(Paragraph("Business Model Opportunities", section_style))

    story.append(Paragraph("Potential Revenue Streams", subsection_style))

    revenue_data = [
        ["Model", "Description", "Target Segment"],
        ["Enterprise License", "Self-hosted with support SLA", "Large enterprises, regulated industries"],
        ["Managed Cloud", "Fully hosted SaaS offering", "Mid-market, startups"],
        ["Usage-Based", "Per-task or per-agent-hour pricing", "Variable workload organizations"],
        ["Professional Services", "Custom agent development, integration", "Enterprises with specific needs"],
        ["Marketplace", "Third-party agent plugins", "Developer ecosystem"]
    ]

    rev_table = Table(revenue_data, colWidths=[1.5*inch, 2.5*inch, 2*inch])
    rev_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), colors.HexColor('#2c5282')),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, -1), 9),
        ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
        ('VALIGN', (0, 0), (-1, -1), 'MIDDLE'),
        ('GRID', (0, 0), (-1, -1), 0.5, colors.HexColor('#cbd5e0')),
        ('ROWBACKGROUNDS', (0, 1), (-1, -1), [colors.HexColor('#f7fafc'), colors.white]),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 8),
        ('TOPPADDING', (0, 0), (-1, -1), 8),
        ('LEFTPADDING', (0, 0), (-1, -1), 6),
    ]))
    story.append(rev_table)

    # Target Customers
    story.append(Paragraph("Target Customer Profiles", subsection_style))

    customers = [
        "<b>Enterprise Development Teams</b> - Organizations with 50+ developers seeking to augment capacity",
        "<b>DevOps/Platform Teams</b> - Teams automating development workflows and CI/CD",
        "<b>Security-Conscious Organizations</b> - Companies in regulated industries needing audit trails",
        "<b>Consulting Firms</b> - Development agencies managing multiple client projects",
        "<b>Open Source Maintainers</b> - Projects seeking automated code review and documentation"
    ]
    for c in customers:
        story.append(Paragraph("• " + c, body_style))

    # Risk Analysis
    story.append(Paragraph("Risk Analysis & Mitigations", section_style))

    risk_data = [
        ["Risk", "Impact", "Mitigation"],
        ["AI model dependency", "High", "Multi-provider support (Anthropic, OpenAI, etc.)"],
        ["Security vulnerabilities", "High", "Age encryption, sandboxed execution, audit logs"],
        ["Market competition", "Medium", "Focus on enterprise features, single-file simplicity"],
        ["Scaling challenges", "Medium", "Kubernetes-native architecture from day one"],
        ["Talent acquisition", "Medium", "Go + React stack has large talent pool"]
    ]

    risk_table = Table(risk_data, colWidths=[1.8*inch, 1*inch, 3.2*inch])
    risk_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), colors.HexColor('#c53030')),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, -1), 9),
        ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
        ('ALIGN', (1, 0), (1, -1), 'CENTER'),
        ('VALIGN', (0, 0), (-1, -1), 'MIDDLE'),
        ('GRID', (0, 0), (-1, -1), 0.5, colors.HexColor('#cbd5e0')),
        ('ROWBACKGROUNDS', (0, 1), (-1, -1), [colors.HexColor('#fff5f5'), colors.white]),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 8),
        ('TOPPADDING', (0, 0), (-1, -1), 8),
        ('LEFTPADDING', (0, 0), (-1, -1), 6),
    ]))
    story.append(risk_table)

    # Strategic Recommendations
    story.append(PageBreak())
    story.append(Paragraph("Strategic Recommendations", section_style))

    story.append(Paragraph("Immediate Priorities", subsection_style))
    recs = [
        "<b>Complete Phase 4</b> - Web dashboard for user-facing monitoring and control",
        "<b>Production pilot</b> - Deploy with design partner for real-world validation",
        "<b>Documentation</b> - API docs, deployment guides, security whitepaper",
        "<b>Benchmark suite</b> - Demonstrate agent effectiveness vs. manual development"
    ]
    for r in recs:
        story.append(Paragraph("1. " + r if recs.index(r) == 0 else f"{recs.index(r)+1}. " + r, body_style))

    story.append(Paragraph("Growth Enablers", subsection_style))
    growth = [
        "<b>Open source core</b> - Consider OSS strategy to build community and trust",
        "<b>Integration ecosystem</b> - GitHub, GitLab, Jira, Linear connectors",
        "<b>Agent marketplace</b> - Enable third-party agent contributions",
        "<b>Enterprise features</b> - SSO, RBAC, compliance certifications"
    ]
    for g in growth:
        story.append(Paragraph("• " + g, body_style))

    # Conclusion
    story.append(Paragraph("Conclusion", section_style))
    story.append(Paragraph(
        "Roea AI represents a significant opportunity in the rapidly growing AI-assisted development market. "
        "The platform addresses critical enterprise needs around security, scalability, and governance that "
        "current single-agent tools cannot meet. With a solid technical foundation already in place and a "
        "clear roadmap for enterprise features, Roea AI is well-positioned to become the orchestration "
        "layer for AI-powered software development.",
        body_style
    ))

    # Key Takeaways Box
    story.append(Spacer(1, 0.2*inch))
    takeaway_data = [
        ['Key Executive Takeaways'],
        ['• Addresses $150B+ market opportunity in AI-augmented development\n'
         '• Unique single-file architecture eliminates operational complexity\n'
         '• Built-in security with age encryption and complete audit trails\n'
         '• Cloud-native, Kubernetes-ready for enterprise scale\n'
         '• Clear development roadmap with phases 1-3 complete']
    ]
    takeaway_table = Table(takeaway_data, colWidths=[6*inch])
    takeaway_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, 0), colors.HexColor('#38a169')),
        ('TEXTCOLOR', (0, 0), (-1, 0), colors.white),
        ('ALIGN', (0, 0), (-1, 0), 'CENTER'),
        ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
        ('FONTSIZE', (0, 0), (-1, 0), 12),
        ('FONTSIZE', (0, 1), (-1, 1), 10),
        ('BACKGROUND', (0, 1), (-1, 1), colors.HexColor('#f0fff4')),
        ('ALIGN', (0, 1), (-1, 1), 'LEFT'),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 12),
        ('TOPPADDING', (0, 0), (-1, -1), 12),
        ('LEFTPADDING', (0, 1), (-1, 1), 15),
        ('BOX', (0, 0), (-1, -1), 1, colors.HexColor('#38a169')),
    ]))
    story.append(takeaway_table)

    # Footer
    story.append(Spacer(1, 0.5*inch))
    story.append(HRFlowable(width="100%", thickness=1, color=colors.HexColor('#cbd5e0')))
    story.append(Spacer(1, 0.1*inch))
    story.append(Paragraph(
        f"Roea AI Executive Summary | Confidential | Generated {datetime.now().strftime('%Y-%m-%d')}",
        ParagraphStyle('Footer', parent=styles['Normal'], fontSize=8, alignment=TA_CENTER, textColor=colors.HexColor('#718096'))
    ))

    doc.build(story)
    print("PDF generated successfully: docs/CEO_Executive_Summary_Roea_AI.pdf")

if __name__ == "__main__":
    create_ceo_document()
